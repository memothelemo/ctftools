use anyhow::Result;

use crate::env::find_tool_executable;
use crate::registry::ToolMetadata;

#[must_use]
pub fn check_tool_install(tool: &ToolMetadata) -> Result<bool> {
    find_tool_executable(tool).map(|path| path.is_some())
}
