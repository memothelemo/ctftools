use anyhow::Result;
use cfg_if::cfg_if;
use std::path::PathBuf;

use crate::registry::ToolMetadata;
use crate::util::which_opt;

mod package_manager;
pub use self::package_manager::*;

/// This function tells whether the program is running in elevation mode.
#[must_use]
pub fn is_running_in_elevation() -> bool {
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
            use tracing::warn;
            match unsafe { windows_impl() } {
                Ok(value) => value,
                Err(error) => {
                    warn!(?error, "Win32 API error occurred while trying to run `is_running_in_evalation` function");
                    false
                }
            }
        } else {
            false
        }
    }
}

pub fn find_tool_executable(tool: &ToolMetadata) -> Result<Option<PathBuf>> {
    // There are ways we can find the tool executable either:
    // 1. By using the `which` operation (from PATH environment variable)
    if let Some(path) = which_opt(&tool.command)? {
        return Ok(Some(path));
    }

    // 2. Checking tool's associated executable (if the operating system is running on Windows)
    #[cfg(target_os = "windows")]
    for path in tool.windows.exec_paths.iter() {
        use anyhow::Context;

        let exists = std::fs::exists(path)
            .with_context(|| format!("failed to find {} executable", path.display()))?;

        if exists {
            return Ok(Some(path.to_path_buf()));
        }
    }

    Ok(None)
}
