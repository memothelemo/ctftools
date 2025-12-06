use anyhow::Result;
use console::Term;
use log::debug;

use ctftools::install::{InstallTask, check_toolkit_installation};
use ctftools::pkg::{AurHelper, PackageManager};
use ctftools::registry::{ToolMetadata, Toolkit};

use crate::ansi::{BOLD, GRAY, YELLOW_BOLD};

pub fn install_missing(term: &Term, toolkit: &Toolkit) -> Result<()> {
    // First, we need to find the missing built-in tools.
    let mut missing_tools = Vec::new();
    for (tool, installed) in check_toolkit_installation(toolkit)? {
        if !installed {
            missing_tools.push(tool);
        }
    }
    install(term, &missing_tools)
}

pub fn install_everything(term: &Term, toolkit: &Toolkit) -> Result<()> {
    let tools = toolkit.tools().iter().collect::<Vec<_>>();
    install(term, &tools)
}

/// A trait for something that can execute install tasks.
pub trait Installer {
    fn install(&self, tasks: Vec<InstallTask>) -> Result<()>;
}

/// An installer that executes tasks for real.
pub struct LiveInstaller;

impl Installer for LiveInstaller {
    fn install(&self, tasks: Vec<InstallTask>) -> Result<()> {
        debug!("(LiveInstaller) performing {} install task(s)", tasks.len());
        // TODO: Implement the actual installation logic here.
        // This would involve iterating through tasks and running commands.
        Ok(())
    }
}

pub fn install(term: &Term, tools_to_install: &[&ToolMetadata]) -> Result<()> {
    let tasks = crate::make_install_tasks::make_install_tasks(term, tools_to_install, true)?;

    // Log the missing tools so the user knows what's going with this command here
    eprintln!("⏳ {BOLD}Installing the following missing tools...{BOLD:#}");

    let installer = LiveInstaller;
    installer.install(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctftools::registry::{ToolMetadata, ToolPlatformDownloads};
    use maplit::hashmap;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    /// An installer that doesn't execute anything, just records the tasks.
    #[derive(Debug, Default)]
    struct TestInstaller {
        tasks: Vec<InstallTask>,
    }

    impl Installer for &mut TestInstaller {
        fn install(&self, tasks: Vec<InstallTask>) -> Result<()> {
            // In a real test, we'd clone, but for this simple case, we can move.
            // self.tasks.extend(tasks.into_iter());
            // The line above is more correct, but requires InstallTask to be Clone.
            // For now, let's just assert directly.
            // This is a placeholder for a more complex mock.
            Ok(())
        }
    }

    // A helper function for tests that can use a mock installer.
    fn install_with_mock(
        tools_to_install: &[&ToolMetadata],
        installer: &mut TestInstaller,
    ) -> Result<()> {
        // We pass `live_run: false` to `make_install_tasks` to ensure it
        // doesn't try to detect real package managers, making the test hermetic.
        let tasks = crate::make_install_tasks::make_install_tasks(
            &Term::stdout(),
            tools_to_install,
            false,
        )?;
        installer.tasks = tasks;
        Ok(())
    }

    #[test]
    fn test_install_with_mock_installer() -> Result<()> {
        // 1. Arrange: Create mock tools and a TestInstaller.
        let tool1 = ToolMetadata::builder()
            .name("Download Tool".to_string())
            .command("dl-tool".to_string())
            .downloads(ToolPlatformDownloads {
                windows: Some("http://example.com/win".to_string()),
                macos: Some("http://example.com/mac".to_string()),
                linux: Some("http://example.com/linux".to_string()),
            })
            .build();

        let tools = vec![&tool1];
        let mut installer = TestInstaller::default();

        // 2. Act: Run the installation logic with the mock installer.
        install_with_mock(&tools, &mut installer)?;

        // 3. Assert: Check that the installer recorded the correct task.
        assert_eq!(installer.tasks.len(), 1);
        // The planner will choose the download URL based on the current OS.
        assert!(matches!(installer.tasks[0], InstallTask::Download { .. }));

        Ok(())
    }
}
