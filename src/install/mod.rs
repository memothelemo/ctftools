use crate::registry::ToolMetadata;

pub mod task;
pub use self::task::*;

/// Represents the result of planning an installation for a single tool.
///
/// This enum indicates whether an [`InstallTask`] could be successfully created
/// or if the tool cannot be installed through any of the available methods.
#[derive(Debug, PartialEq, Eq)]
pub enum InstallPlanResult<'a> {
    /// An installation task was successfully created.
    Task(InstallTask),

    /// The tool could not be installed, with a reason.
    CannotInstall(&'a ToolMetadata, InstallTaskError),
}
