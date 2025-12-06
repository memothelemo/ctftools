use anstream::eprintln;
use anyhow::Result;
use console::Term;
use log::debug;
use log::warn;

use crate::cli::ansi::*;
use crate::env::Environment;
use crate::install::InstallPlanResult;
use crate::registry::{ToolMetadata, Toolkit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallGoal<'t> {
    Everything,
    Missing,
    Specific(&'t [ToolMetadata]),
}

pub fn install(
    env: &dyn Environment,
    goal: InstallGoal<'_>,
    stderr: &Term,
    toolkit: &Toolkit,
) -> Result<()> {
    // If our toolkit is empty, then tell the user about it?
    if toolkit.tools().is_empty() {
        println!("All required tool(s) are empty.");
        return Ok(());
    }

    if env.pkg_manager().is_none() && env.is_live() {
        warn!(
            "It is recommened to install a package manager to automate \
        the process of installing the tools you need. Please install your \
        preferred package manager in your current operating system."
        );
    }

    debug!(
        "running in elevation: {}",
        crate::util::running_in_elevation()
    );

    let outcomes = match goal {
        InstallGoal::Everything => env.plan_install_tools(toolkit.tools()),
        InstallGoal::Missing => env.plan_install_missing_tools(toolkit)?,
        InstallGoal::Specific(tools) => env.plan_install_tools(tools),
    };
    debug!(
        "found {} potential tool(s) that can be installed",
        outcomes.len()
    );

    // Filter out the outcome that thrown an error so we only have
    // successfully made tasks left.
    stderr.hide_cursor()?;

    let mut tasks = Vec::new();
    for outcome in outcomes {
        match outcome {
            InstallPlanResult::Task(task) => {
                debug!("added task to install: {task:?}");
                tasks.push(task);
            }
            InstallPlanResult::CannotInstall(tool, error) => {
                eprintln!(
                    "{YELLOW_BOLD}⚠️ Could not install '{name}': {error}{YELLOW_BOLD:#}",
                    name = tool.name
                );
            }
        }
    }

    // If we already installed all of the tools in the toolkit,
    // we can print out the message to the user.
    if tasks.is_empty() {
        if toolkit.tools().is_empty() {
            println!("There are no tools requiring you to install.");
        } else {
            println!("All required tool(s) are already installed.");
        }
        return Ok(());
    }

    // // Log the missing tools so the user knows what's going with this command here
    // debug!("installing {} tool(s)", tasks.len());
    // eprintln!("⏳ {BOLD}Installing the following missing tools...{BOLD:#}");
    // for task in tasks.iter() {
    //     println!("{GRAY}* {}{GRAY:#}", task.tool_name());
    // }
    // eprintln!();

    // let mut tracker = env.run_install_tasks(tasks)?;
    // crate::signals::lock_terminate_signals();

    // while let Some(message) = tracker.next() {
    //     match message {
    //         InstallProgress::Command { text, tool_name } => {
    //             eprintln!("{BOLD}Installing {tool_name}{BOLD:#}: {GRAY}{text}{GRAY:#}");
    //         }
    //         InstallProgress::Success { tool_name, elapsed } => {
    //             eprintln!(
    //                 "{GREEN}{BOLD}✅ Successfully installed {tool_name} \
    //                 ({elapsed:.2?}){BOLD:#}{GREEN:#}"
    //             );
    //         }
    //         InstallProgress::Error { error, tool_name } => {
    //             bail!(
    //                 "Failed to install {tool_name} (you may \
    //                 want to install it manually instead): {error}"
    //             );
    //         }
    //     };
    // }

    // stderr.show_cursor()?;
    // crate::signals::unlock_terminate_signals();
    Ok(())
}
