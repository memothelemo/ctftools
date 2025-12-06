use anyhow::Result;
use console::Term;

use ctftools::install::InstallTask;
use ctftools::install::planner::{InstallPlanner, PlanningOutcome};
use ctftools::registry::{ToolMetadata, Toolkit};

use crate::ansi::YELLOW_BOLD;

pub fn make_install_tasks(
    term: &Term,
    tools_to_install: &[&ToolMetadata],
    live_run: bool,
) -> Result<Vec<InstallTask>> {
    term.hide_cursor()?;

    let planner = if live_run {
        InstallPlanner::new()?
    } else {
        InstallPlanner::without_package_managers()
    };

    if !planner.has_package_manager() {
        eprintln!(
            "{YELLOW_BOLD}⚠️ It is recommened to install a package manager to automate \
        the process of installing the tools you need. Please install your \
        preferred package manager in your current operating system.{YELLOW_BOLD:#}"
        );
    }

    let mut tasks = Vec::new();
    for outcome in planner.plan_installs(tools_to_install) {
        match outcome {
            PlanningOutcome::Task(task) => tasks.push(task),
            PlanningOutcome::CannotInstall(tool, error) => {
                eprintln!(
                    "{YELLOW_BOLD}⚠️ Could not install '{name}': {error}{YELLOW_BOLD:#}",
                    name = tool.name
                );
            }
        }
    }

    Ok(tasks)
}

pub fn serialize_install_tasks(term: &Term, toolkit: &Toolkit) -> Result<()> {
    let tools_to_install = toolkit.tools().iter().collect::<Vec<_>>();
    let tasks = make_install_tasks(term, &tools_to_install, false)?;
    let serialized = serde_json::to_string_pretty(&tasks)?;
    println!("{serialized}");

    Ok(())
}
