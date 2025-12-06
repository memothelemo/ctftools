use anyhow::Result;
use cfg_if::cfg_if;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn run_cmd(exec: PathBuf, args: Vec<String>) -> Command {
    let mut cmd = std::process::Command::new(exec);
    cmd.args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    cmd
}

/// Returns a human-readable representation of a command.
///
/// The returned string contains the program path (or name) followed by
/// its arguments, separated by spaces.
#[must_use]
pub fn cmd_display(cmd: &Command) -> String {
    format!(
        "{} {}",
        cmd.get_program().to_string_lossy(),
        cmd.get_args()
            .map(|v| v.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    )
}

/// Returns the result of [`which::which`] but it returns
/// an optional value whether the specified name exists or not.
pub fn which_opt<T: AsRef<OsStr>>(name: T) -> Result<Option<PathBuf>> {
    match which::which(name) {
        Ok(okay) => Ok(Some(okay)),
        Err(which::Error::CannotFindBinaryPath) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// Checks if the current operating system allows escalating
/// process privileges on demand.
#[must_use]
pub fn supports_privilege_escalation() -> bool {
    cfg!(target_os = "linux")
}

/// This function tells whether the program is running in elevation mode.
#[must_use]
pub fn running_in_elevation() -> bool {
    #[cfg(unix)]
    fn unix_impl() -> bool {
        use sudo::RunningAs;
        match sudo::check() {
            RunningAs::Root | RunningAs::Suid => true,
            RunningAs::User => false,
        }
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    #[cfg(windows)]
    unsafe fn windows_impl() -> Result<bool> {
        // https://stackoverflow.com/a/95918/23025722
        use anyhow::Context;
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::Security::{
            GetTokenInformation, TOKEN_ELEVATION, TOKEN_READ, TokenElevation,
        };
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        let mut handle_token = HANDLE(0 as _);
        OpenProcessToken(GetCurrentProcess(), TOKEN_READ, &mut handle_token)
            .context("could not open process token for the current process")?;

        let mut size_returned = 0u32;
        let mut elevation_info: TOKEN_ELEVATION = std::mem::zeroed();

        if let Err(error) = GetTokenInformation(
            handle_token,
            TokenElevation,
            Some(&mut elevation_info as *mut _ as *mut _),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut size_returned as *mut _,
        ) {
            CloseHandle(handle_token).context("could not close process token")?;
            return Err(error.into());
        }

        CloseHandle(handle_token).context("could not close process token")?;
        Ok(elevation_info.TokenIsElevated != 0)
    }

    cfg_if! {
        if #[cfg(unix)] {
            unix_impl()
        } else if #[cfg(windows)] {
            use log::warn;
            match unsafe { windows_impl() } {
                Ok(value) => value,
                Err(error) => {
                    warn!("Win32 API error occurred while trying to run `is_running_in_evalation` function: {error}");
                    false
                }
            }
        } else {
            false
        }
    }
}
