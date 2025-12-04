use anyhow::{Context, Result, bail};
use cfg_if::cfg_if;
use log::debug;
use std::path::PathBuf;
use std::process::Command;

use crate::util::{is_running_in_elevation, pretty_cmd, supports_privilege_escalation};

mod unix;
mod windows;

#[must_use]
pub fn make_cmd(exec: PathBuf, arguments: Vec<String>, sudo: bool) -> Command {
    // Do not include sudo if we're running in an non-Unix system
    let sudo_bin = if cfg!(unix) && sudo {
        crate::util::which_opt("sudo").ok().flatten()
    } else {
        None
    };

    let mut cmd = if let Some(sudo_bin) = sudo_bin {
        let mut cmd = Command::new(sudo_bin);
        cmd.arg(exec);
        cmd
    } else {
        Command::new(exec)
    };

    cmd.args(arguments);
    cmd
}

#[derive(Debug, Clone, Copy)]
pub struct Status(pub i32);

impl std::fmt::Display for Status {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl std::error::Error for Status {}

impl Status {
    pub fn code(self) -> i32 {
        self.0
    }

    pub fn success(self) -> Result<i32, Status> {
        if self.0 == 0 { Ok(0) } else { Err(self) }
    }
}

/// Prepares a command for execution, verifying if it requires elevated privileges.
pub fn run_cmd(cmd: &mut Command, needs_privilege: bool) -> Result<()> {
    debug!(
        "Preparing to run command: {:?}; requires elevation: {}",
        cmd, needs_privilege
    );

    // Check if the command requires elevated privileges.
    //
    // If so, verify whether the current process is running with sufficient privileges.
    //
    // If the process is not elevated and the OS does not support privilege escalation,
    // return an informative error message prompting the user to run with elevated privileges.
    if needs_privilege && !is_running_in_elevation() && !supports_privilege_escalation() {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                bail!("Please run your terminal as administrator to allow memotools to install missing tools.");
            } else {
                bail!("Please run this command with elevated privileges to install missing tools.");
            }
        }
    }

    wait_for_status(cmd)?
        .success()
        .with_context(|| make_command_error(cmd))?;

    Ok(())
}

fn wait_for_status(cmd: &mut Command) -> Result<Status> {
    cmd.status()
        .map(|s| Status(s.code().unwrap_or(1)))
        .with_context(|| make_command_error(cmd))
}

fn make_command_error(cmd: &Command) -> String {
    format!("failed to run: {}", pretty_cmd(cmd))
}
