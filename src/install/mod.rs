use crate::registry::ToolMetadata;
use std::time::Duration;

pub mod live;
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

    /// Interrupt signal has been triggered. This is just a first
    /// warning message reminding the user that this process will
    /// be interrupted if triggered again.
    InterruptFirstWarning,

    /// The installation process is interrupted.
    Interrupted,

    /// A tool was successfully installed.
    Success {
        /// How long it takes to install a tool.
        elapsed: Duration,

        /// Associated tool that was successfully installed.
        tool_name: String,
    },
}

// #[derive(Debug)]
// pub struct InstallTracker {
//     recv: mpsc::Receiver<InstallProgress>,
// }

// impl InstallTracker {
//     #[must_use]
//     pub(crate) fn new() -> (Self, mpsc::Sender<InstallProgress>) {
//         let (tx, rx) = mpsc::channel();
//         let tracker = Self { recv: rx };
//         (tracker, tx)
//     }

//     #[allow(clippy::should_implement_trait)]
//     #[must_use]
//     pub fn next(&mut self) -> Option<InstallProgress> {
//         self.recv.recv().ok()
//     }
// }
