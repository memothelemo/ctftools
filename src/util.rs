use anyhow::Result;
use cfg_if::cfg_if;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

/// Returns true if the current process was most likely started by a user
/// double-clicking an application icon (i.e. launched from a graphical file
/// manager) rather than being started from an interactive terminal/shell.
///
/// As of writing this function, there's no implementation on Unix systems
/// so it assumes that this process is started by the terminal.
///
/// Note: This function should be used only for UX decisions (e.g. whether to
/// show GUI dialogs or spawn consoles) and never for security-sensitive logic.
#[must_use]
pub fn started_by_double_click() -> bool {
    #[allow(unsafe_op_in_unsafe_fn)]
    #[cfg(windows)]
    unsafe fn windows_impl() -> Result<bool> {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32, Process32First, Process32Next,
            TH32CS_SNAPPROCESS,
        };

        unsafe fn get_process_entry(pid: u32) -> Result<Option<PROCESSENTRY32>> {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;

            let mut process_entry: PROCESSENTRY32 = std::mem::zeroed();
            process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
            process_entry.szExeFile = [0; 260];

            Process32First(snapshot, &mut process_entry as *mut _)?;

            loop {
                if process_entry.th32ProcessID == pid {
                    CloseHandle(snapshot)?;
                    return Ok(Some(process_entry));
                }

                if Process32Next(snapshot, &mut process_entry as *mut _).is_err() {
                    CloseHandle(snapshot)?;
                    return Ok(None);
                }
            }
        }

        // explorer.exe
        let explorer_exe: [i8; 12] = [101, 120, 112, 108, 111, 114, 101, 114, 46, 101, 120, 101];
        let Some(entry) = get_process_entry(std::process::id())? else {
            return Ok(false);
        };

        match get_process_entry(entry.th32ParentProcessID)? {
            None => Ok(false),
            Some(e) => Ok(e.szExeFile[0..12] == explorer_exe),
        }
    }

    cfg_if! {
        if #[cfg(windows)] {
            use log::warn;
            match unsafe { windows_impl() } {
                Ok(value) => value,
                Err(error) => {
                    warn!("Win32 API error occurred while trying to run `started_by_double_click` function: {error}");
                    false
                }
            }
        } else {
            false
        }
    }
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
