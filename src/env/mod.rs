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

    #[cfg(windows)]
    fn windows_impl() -> bool {
        todo!()
    }

    cfg_if! {
        if #[cfg(unix)] {
            unix_impl()
        } else if #[cfg(windows)] {
            windows_impl()
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
        let exists = std::fs::exists(path)
            .with_context(|| format!("failed to find {} executable", path.display()))?;

        if exists {
            return Ok(Some(path.to_path_buf()));
        }
    }

    Ok(None)
}
