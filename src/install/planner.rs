use anyhow::Result;
use log::debug;

use crate::install::{InstallTask, InstallTaskError};
use crate::pkg::{AurHelper, PackageManager};
use crate::registry::ToolMetadata;

/// Represents the outcome of planning an installation for a single tool.
#[derive(Debug)]
pub enum PlanningOutcome<'a> {
    /// A task was successfully created.
    Task(InstallTask),
    /// The tool could not be installed, with a reason.
    CannotInstall(&'a ToolMetadata, InstallTaskError),
}

/// Creates a plan of `InstallTask`s for a list of tools.
///
/// This component is responsible for determining *how* tools should be
/// installed, but does not execute anything.
pub struct InstallPlanner {
    pkg_manager: Option<(PackageManager, std::path::PathBuf)>,
    aur_helper: Option<(AurHelper, std::path::PathBuf)>,
}

impl InstallPlanner {
    /// Creates a new `InstallPlanner`, detecting the available system package managers.
    pub fn new() -> Result<Self> {
        Ok(Self {
            pkg_manager: PackageManager::detect()?,
            aur_helper: AurHelper::detect()?,
        })
    }

    /// Creates a new `InstallPlanner` without detecting package managers.
    /// This is useful for testing or dry-runs where no real installation will occur.
    #[must_use]
    pub fn without_package_managers() -> Self {
        Self {
            pkg_manager: None,
            aur_helper: None,
        }
    }

    /// Returns `true` if a primary package manager (like apt, dnf, brew) was found.
    pub fn has_package_manager(&self) -> bool {
        self.pkg_manager.is_some()
    }

    /// Generates a vector of `PlanningOutcome` for the given tools.
    pub fn plan_installs<'a>(
        &self,
        tools_to_install: &[&'a ToolMetadata],
    ) -> Vec<PlanningOutcome<'a>> {
        let mut outcomes = Vec::new();
        for tool in tools_to_install {
            let command = tool.command.to_string();
            let outcome = self.plan_for_tool(tool);
            debug!("created install plan for {command:?}; outcome = {outcome:?}");
            outcomes.push(outcome);
        }
        outcomes
    }

    fn plan_for_tool<'a>(&self, tool: &'a ToolMetadata) -> PlanningOutcome<'a> {
        if let Some((pkg_manager, path_to_pkgm)) = self.pkg_manager.clone() {
            match InstallTask::from_package_manager(pkg_manager, path_to_pkgm, tool) {
                Ok(mut task) => {
                    // If it's an AUR task, try to refine it with the AUR helper.
                    if pkg_manager == PackageManager::Pacman
                        && matches!(task, InstallTask::AUR { .. })
                        && let Some((aur_helper, path_to_arh)) = self.aur_helper.clone()
                    {
                        if let InstallTask::AUR { package_name } = task {
                            task = InstallTask::from_aur(aur_helper, path_to_arh, package_name);
                        }
                    }
                    return PlanningOutcome::Task(task);
                }
                Err(e @ InstallTaskError::PackageNotFound { .. }) => {
                    // This isn't a fatal error; we can try other methods.
                    debug!(
                        "Package not found for {}: {e}, trying downloads.",
                        tool.name
                    );
                }
                Err(e) => return PlanningOutcome::CannotInstall(tool, e),
            };
        }

        // Fallback to downloads
        match InstallTask::from_downloads(tool) {
            Ok(task) => PlanningOutcome::Task(task),
            Err(e) => PlanningOutcome::CannotInstall(tool, e),
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: Add unit tests for InstallPlanner
}
