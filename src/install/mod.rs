use anyhow::Result;
use std::path::PathBuf;

use crate::registry::{ToolMetadata, Toolkit};
use crate::util::which_opt;

pub mod task;
pub use self::task::{InstallTask, InstallTaskError};

/// Checks which tools in a `Toolkit` are installed on the system.
///
/// It returns a vector of tuples, where each tuple contains:
/// - a reference to the [tool's metadata](ToolMetadata)
/// - a boolean indicating whether the tool's executable could be found or installed
pub fn check_toolkit_installation(toolkit: &Toolkit) -> Result<Vec<(&ToolMetadata, bool)>> {
    let iter = toolkit.tools().iter();
    iter.map(|tool| {
        let installed = find_tool_executable(tool)?.is_some();
        Ok::<_, _>((tool, installed))
    })
    .collect()
}

/// Attempts to locate the executable for a specific tool described by [`ToolMetadata`.]
///
/// The lookup strategy is:
/// 1. Try to find the command on the system `PATH`.
/// 2. On Windows, also check any additional executable paths associated
///    with the tool's metadata.
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
