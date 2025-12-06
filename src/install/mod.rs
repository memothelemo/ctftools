use std::sync::mpsc;

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

#[derive(Debug)]
pub enum InstallProgress {
    Command { text: String },
    Error(anyhow::Error),
}

#[derive(Debug)]
pub struct InstallTracker {
    recv: mpsc::Receiver<InstallProgress>,
}

impl InstallTracker {
    #[must_use]
    pub(crate) fn new() -> (Self, mpsc::Sender<InstallProgress>) {
        let (tx, rx) = mpsc::channel();
        let tracker = Self { recv: rx };
        (tracker, tx)
    }

    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn next(&mut self) -> Option<InstallProgress> {
        self.recv.recv().ok()
    }
}
