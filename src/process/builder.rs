// Copied from: https://github.com/rust-lang/cargo/blob/973538787ee13a52199c07ba9b14135e4cac19e7/crates/cargo-util/src/process_builder.rs
// Licensed under MIT/Apache-2.0
use anyhow::{Context, Result};
use libc::{SIGINT, SIGTERM};
use shell_escape::escape;
use signal_hook::flag as signal_flag;

use std::ffi::{OsStr, OsString};
use std::iter::once;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::process::ProcessError;

/// A custom builder object for building an external process,
/// similar to [`std::process::Command`].
#[derive(Debug, Clone)]
pub struct ProcessBuilder {
    /// A binary file to execute.
    program: PathBuf,

    /// A list of arguments to pass to the program.
    args: Vec<OsString>,

    /// A list of wrappers that wrap the original program.
    ///
    /// The last entry is the outermost wrapper.
    wrappers: Vec<PathBuf>,
}

impl ProcessBuilder {
    /// Creates a new [`ProcessBuilder`] with the given executable binary path.
    #[must_use]
    pub fn new<T: Into<PathBuf>>(cmd: T) -> Self {
        Self {
            program: cmd.into(),
            args: Vec::new(),
            wrappers: Vec::new(),
        }
    }

    /// Adds `arg` to the args list.
    pub fn arg<T: AsRef<OsStr>>(&mut self, arg: T) -> &mut ProcessBuilder {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    /// Adds multiple `args` to the args list.
    pub fn args<T: AsRef<OsStr>>(&mut self, args: &[T]) -> &mut ProcessBuilder {
        let iter = args.iter().map(|t| t.as_ref().to_os_string());
        self.args.extend(iter);
        self
    }

    /// Gets the executable name of the process to run.
    ///
    /// If it is wrapped, then the last entry of the wrappers
    /// will be the outermost while the innermost will be the
    /// base program.
    #[must_use]
    pub fn get_program(&self) -> &OsStr {
        self.wrappers
            .last()
            .map(|v| v.as_os_str())
            .unwrap_or(self.program.as_os_str())
    }

    /// Gets the program arguments.]
    pub fn get_args(&self) -> impl Iterator<Item = &OsStr> {
        self.wrappers
            .iter()
            .rev()
            .map(|v| v.as_os_str())
            .chain(once(self.program.as_os_str()))
            .chain(self.args.iter().map(|v| v.as_os_str()))
            .skip(1)
    }

    /// Wraps an existing command with the provided wrapper,
    /// if it is present and valid.
    pub fn wrap<T: Into<PathBuf>>(&mut self, wrapper: Option<T>) {
        if let Some(wrapper) = wrapper {
            let wrapper = wrapper.into();
            if !wrapper.as_os_str().is_empty() {
                self.wrappers.push(wrapper.to_path_buf());
            }
        }
    }
}

impl ProcessBuilder {
    /// Executes the process, returning the stdio output, or an error if non-zero exit status.
    pub fn exec_with_output(&self) -> Result<Output> {
        let output = self.output()?;
        if output.status.success() {
            Ok(output)
        } else {
            Err(ProcessError::new(
                &format!("process didn't exit successfully: {}", self),
                Some(output.status),
                Some(&output),
            )
            .into())
        }
    }

    /// Like [`Command::output`] but with a better error message.
    pub fn output(&self) -> Result<Output> {
        self.output_inner()
            .with_context(|| ProcessError::could_not_execute(self))
    }

    /// Inner function of [`ProcessBuilder::output`].
    fn output_inner(&self) -> std::io::Result<Output> {
        let mut cmd = self.build_command();
        match piped(&mut cmd, false).spawn() {
            Ok(child) => child.wait_with_output(),
            Err(e) => Err(e),
        }
    }

    /// Converts [`ProcessBuilder`] into a [`std::process::Command`].
    #[must_use]
    pub fn build_command(&self) -> Command {
        let mut command = {
            let mut iter = self.wrappers.iter().rev().chain(once(&self.program));
            let mut cmd = Command::new(iter.next().expect("at least one `program` exists"));
            cmd.args(iter);
            cmd
        };

        // Then, we can insert arguments
        for arg in self.args.iter() {
            command.arg(arg);
        }

        command
    }
}

#[derive(Debug)]
pub enum LockedNotification {
    FirstWarning,
    Interrupted,
}

impl ProcessBuilder {
    /// Executes the process, returning the stdio output, or an error
    /// if non-zero exit status but it blocks all of the exit signals
    /// while the process is running (unless if it is triggered twice).
    pub fn exec_locked(&self, notification: &mut dyn FnMut(LockedNotification)) -> Result<Output> {
        let mut child = self
            .build_command()
            .spawn()
            .with_context(|| ProcessError::could_not_execute(self))?;

        // Loop until the child process exits or got triggered by
        // one of the signals TWICE, checking for signals periodically.
        let mut times_triggered = 0usize;
        let mut last_triggered = Instant::now();

        let exit_flag = Arc::new(AtomicBool::new(false));
        let sigint_id = signal_flag::register(SIGINT, Arc::clone(&exit_flag))?;
        let sigterm_id = signal_flag::register(SIGTERM, Arc::clone(&exit_flag))?;

        loop {
            // Check if an interrupt signal has been received.
            if exit_flag.load(Ordering::Relaxed) {
                if last_triggered.elapsed() >= Duration::from_secs(5) && times_triggered != 0 {
                    times_triggered = 0;
                }

                last_triggered = Instant::now();
                times_triggered += 1;

                if times_triggered == 1 {
                    notification(LockedNotification::FirstWarning);
                    continue;
                }

                // The user has requested to terminate the installation.
                // Kill the child process and report the interruption.
                child
                    .kill()
                    .context("failed to kill installation process")?;

                child
                    .wait()
                    .context("failed to wait for killed installation process")?;

                notification(LockedNotification::Interrupted);

                // Unregister all of the signal handlers
                signal_hook::low_level::unregister(sigint_id);
                signal_hook::low_level::unregister(sigterm_id);

                let output = child.wait_with_output()?;
                return Ok(output);
            }

            // Check if the child process has finished without blocking.
            match child.try_wait()? {
                Some(status) => {
                    // The process has finished. Unregister the signal handlers.
                    signal_hook::low_level::unregister(sigint_id);
                    signal_hook::low_level::unregister(sigterm_id);

                    if !status.success() {
                        return Err(ProcessError::new(
                            &format!("process didn't exit successfully: {self}"),
                            Some(status),
                            None,
                        )
                        .into());
                    }

                    // Report success.
                    let output = child.wait_with_output()?;
                    return Ok(output);
                }
                None => {
                    // The process is still running. Wait a bit before checking again.
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
    }
}

impl std::fmt::Display for ProcessBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_program().display())?;
        for arg in self.get_args() {
            write!(f, " {}", escape(arg.to_string_lossy()))?;
        }
        Ok(())
    }
}

/// Creates new pipes for stderr, stdout, and optionally stdin.
fn piped(cmd: &mut Command, pipe_stdin: bool) -> &mut Command {
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(if pipe_stdin {
            Stdio::piped()
        } else {
            Stdio::null()
        })
}

#[cfg(test)]
mod tests {
    use crate::process::ProcessBuilder;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_display_fmt() {
        let builder = ProcessBuilder::new("/usr/bin/sudo");
        assert_eq!(format!("{builder}"), "/usr/bin/sudo");

        let mut builder = ProcessBuilder::new("/usr/bin/sudo");
        builder.arg("-a");
        assert_eq!(format!("{builder}"), "/usr/bin/sudo -a");

        let mut builder = ProcessBuilder::new("/usr/bin/sudo");
        builder.arg("-a");
        builder.arg("-b");
        assert_eq!(format!("{builder}"), "/usr/bin/sudo -a -b");

        let mut builder = ProcessBuilder::new("/usr/bin/pacman");
        builder.wrap(Some("/usr/bin/sudo"));
        builder.arg("--hello");

        assert_eq!(
            format!("{builder}"),
            "/usr/bin/sudo /usr/bin/pacman --hello"
        );
    }
}
