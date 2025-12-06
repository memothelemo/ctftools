use anyhow::Result;
use log::debug;
use std::path::PathBuf;

use crate::install::{InstallPlanResult, InstallTask, InstallTaskError, InstallTracker};
use crate::pkg::{AurHelper, PackageManager};
use crate::registry::{ToolMetadata, Toolkit};

pub mod live;
pub mod mock;

pub use self::live::LiveEnvironment;
pub use self::mock::MockEnvironment;

/// Defines an abstraction over the host system's environment.
///
/// This trait allows for querying system properties like available package managers
/// and creating installation plans without being tied to a specific "live" environment.
///
/// This is crucial for testing and dry-runs, as it allows substituting a mock
/// environment.
pub trait Environment: std::fmt::Debug {
    /// Whether this environment is running in live.
    #[must_use]
    fn is_live(&self) -> bool {
        false
    }

    /// Gets the current [package manager] along with its binary path of the environment.
    ///
    /// [package manager]: PackageManager
    #[must_use]
    fn pkg_manager(&self) -> Option<(PackageManager, PathBuf)>;

    /// Gets the current [AUR helper] along with its binary path of the environment.
    ///
    /// This is only relevant on Arch Linux systems.
    ///
    /// [AUR helper]: AurHelper
    #[must_use]
    fn aur_helper(&self) -> Option<(AurHelper, PathBuf)>;

    /// Checks which tools in a [`Toolkit`] are installed in the environment.
    ///
    /// It returns a vector of tuples, where each tuple contains:
    /// - a reference to the [tool's metadata]
    /// - a boolean indicating whether the tool's executable could be found or installed
    ///
    /// [environment]: Environment
    /// [tool's metadata]: ToolMetadata
    fn check_toolkit_installation<'t>(
        &self,
        toolkit: &'t Toolkit,
    ) -> Result<Vec<(&'t ToolMetadata, bool)>> {
        let iter = toolkit.tools().iter();
        iter.map(|tool| {
            let installed = self.find_tool_executable(tool)?.is_some();
            Ok::<_, _>((tool, installed))
        })
        .collect()
    }

    /// Attempts to locate the executable for a specific tool.
    ///
    /// Lookup strategies may differ depending on the true value
    /// based on [environment] trait.
    ///
    /// Implementations of this method define the strategy for finding a tool,
    /// such as checking the system's `PATH` or other well-known locations.
    fn find_tool_executable(&self, tool: &ToolMetadata) -> Result<Option<PathBuf>>;

    /// Creates an installation plan for all tools in a [`Toolkit`] that are not yet installed.
    ///
    /// This method first checks the installation status of all tools and then
    /// generates a plan for the missing ones.
    fn plan_install_missing_tools<'t>(
        &self,
        toolkit: &'t Toolkit,
    ) -> Result<Vec<InstallPlanResult<'t>>> {
        let mut outcomes = Vec::new();
        for (tool, installed) in self.check_toolkit_installation(toolkit)? {
            if !installed {
                outcomes.push(self.plan_install_tool(tool));
            }
        }
        Ok(outcomes)
    }

    /// Creates an installation plan for a given slice of tools.
    ///
    /// This method iterates through the provided tools and determines the best
    /// installation strategy for each one.
    fn plan_install_tools<'t>(
        &self,
        tools_to_install: &'t [ToolMetadata],
    ) -> Vec<InstallPlanResult<'t>> {
        let mut outcomes = Vec::new();
        for tool in tools_to_install {
            let command = tool.command.to_string();
            let outcome = self.plan_install_tool(tool);
            debug!("created install plan for {command:?}; outcome = {outcome:?}");
            outcomes.push(outcome);
        }
        outcomes
    }

    /// Creates an installation plan for a single tool.
    ///
    /// This is the core planning logic, which attempts to create an [`InstallTask`]
    /// by first checking for a package manager and then falling back to direct
    /// downloads if necessary.
    fn plan_install_tool<'t>(&self, tool: &'t ToolMetadata) -> InstallPlanResult<'t> {
        if let Some((pkg_manager, path_to_pkgm)) = self.pkg_manager().clone() {
            match InstallTask::from_package_manager(pkg_manager, path_to_pkgm, tool) {
                Ok(mut task) => {
                    // If it's an AUR task, try to refine it with the AUR helper.
                    if pkg_manager == PackageManager::Pacman
                        && matches!(task, InstallTask::AUR { .. })
                        && let Some((aur_helper, path_to_arh)) = self.aur_helper().clone()
                        && let InstallTask::AUR {
                            package_name,
                            tool_name,
                        } = task
                    {
                        task =
                            InstallTask::from_aur(aur_helper, path_to_arh, package_name, tool_name);
                    }
                    return InstallPlanResult::Task(task);
                }
                Err(e @ InstallTaskError::PackageNotFound { .. }) => {
                    // This isn't a fatal error; we can try other methods.
                    debug!(
                        "package not found for {}: {e}, trying downloads.",
                        tool.name
                    );
                }
                Err(e) => return InstallPlanResult::CannotInstall(tool, e),
            };
        }

        // Fallback to downloads
        match InstallTask::from_downloads(tool) {
            Ok(task) => InstallPlanResult::Task(task),
            Err(e) => InstallPlanResult::CannotInstall(tool, e),
        }
    }

    fn run_install_tasks(&self, tasks: Vec<InstallTask>) -> Result<InstallTracker>;
}

#[cfg(test)]
mod tests {
    use crate::env::Environment;
    use static_assertions::assert_obj_safe;

    assert_obj_safe!(Environment);
}
