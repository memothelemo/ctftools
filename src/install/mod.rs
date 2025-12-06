use std::{sync::mpsc, time::Duration};

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
    /// This indicates that ctftools executes a package manager
    /// command that allows for a tool to be installed.
    Command {
        /// What is the command initiated in order to install
        /// a tool to a package manager or AUR helper?
        text: String,

        /// Associated tool that will be installed.
        tool_name: String,
    },

    /// A tool was successfully installed.
    Success {
        /// How long it takes to install a tool.
        elapsed: Duration,

        /// Associated tool that was successfully installed.
        tool_name: String,
    },

    /// An error occurred while trying to installing a tool.
    Error {
        /// Source of the error
        error: anyhow::Error,

        /// Associated tool that got an error
        tool_name: String,
    },
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
