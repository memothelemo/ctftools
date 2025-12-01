use std::time::Duration;

use anstream::eprintln;
use anyhow::Context;
use anyhow::Result;
use console::Term;
use log::debug;
use log::warn;

use crate::cli::ansi::*;
use crate::cli::debug_enabled;
use crate::env::Environment;
use crate::install::InstallPlanResult;
use crate::install::InstallProgress;
use crate::registry::{ToolMetadata, Toolkit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallGoal<'t> {
    Everything,
    Missing,
    Specific(&'t [ToolMetadata]),
}

#[derive(Debug)]
enum InstallResult {
    Successful { elapsed: Duration },
    Failed { error: anyhow::Error },
    Skipped,
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
        print!("✅ {GREEN}{BOLD}");
        if toolkit.tools().is_empty() {
            print!("There are no tools requiring you to install.");
        } else {
            print!("All required tool(s) are already installed.");
        }
        println!("{GREEN:#}{BOLD:#}");
        return Ok(());
    }

    // Log the missing tools so the user knows what's going with this command here
    debug!("installing {} tool(s)", tasks.len());
    eprintln!("⏳ {BOLD}Installing the following missing tools...{BOLD:#}");
    for task in tasks.iter() {
        println!("{GRAY}* {}{GRAY:#}", task.tool_name());
    }
    eprintln!();

    let mut got_interrupted = false;
    let mut results = tasks
        .iter()
        .map(|task| (task, InstallResult::Skipped))
        .collect::<Vec<_>>();

    for (task, result) in results.iter_mut() {
        if got_interrupted {
            break;
        }

        let mut progress_handler = &mut |progress: InstallProgress| {
            match progress {
                InstallProgress::Interrupted => {
                    #[allow(unused)]
                    {
                        got_interrupted = true;
                    }
                }
                InstallProgress::InterruptFirstWarning => {
                    eprintln!(
                        "{YELLOW_BOLD}⚠️ Triggered interrupt signal. Trigger again \
                    to stop the installation process{YELLOW_BOLD:#}",
                    );
                }
                InstallProgress::Command { text, tool_name } => {
                    eprintln!("{BOLD}Installing {tool_name}{BOLD:#}: {GRAY}{text}{GRAY:#}");
                }
                InstallProgress::Success { elapsed, .. } => {
                    *result = InstallResult::Successful { elapsed };
                }
                _ => panic!("unimplemented progress: {progress:?}"),
            };
        };

        let tool_name = task.tool_name();
        let output = env
            .run_install_task(task, &mut progress_handler)
            .with_context(|| {
                format!(
                    "Failed to install {tool_name} (you may want to \
                    install it manually instead)"
                )
            });

        if let Err(error) = output {
            *result = InstallResult::Failed { error };
            break;
        }

        if !debug_enabled() {
            stderr.clear_screen()?;
        }
    }

    let successfully_installed = results
        .iter()
        .all(|(_, result)| matches!(result, InstallResult::Successful { .. }));

    if successfully_installed {
        eprintln!("✅ {GREEN}{BOLD}Successfully installed the following tools!{BOLD:#}{GREEN:#}");
    } else {
        eprintln!("😭 {RED}{BOLD}Failed to install one of the following tools!{BOLD:#}{RED:#}");
    }

    let mut captured_error = None;
    for (task, result) in results.into_iter() {
        match result {
            InstallResult::Successful { elapsed } => {
                println!(
                    "{GRAY}* {BOLD}{}{BOLD:#} ({elapsed:.2?}){GRAY:#}",
                    task.tool_name()
                );
            }
            InstallResult::Failed { error } => {
                println!(
                    "{GRAY}* {RED}{BOLD}{} (failed){BOLD:#}{RED:#} {GRAY:#}",
                    task.tool_name()
                );
                captured_error = Some(error);
            }
            InstallResult::Skipped => {
                println!(
                    "{GRAY}*{GRAY:#} {RED}{BOLD}{} (skipped){RED:#}{BOLD:#}",
                    task.tool_name()
                );
            }
        }
    }

    if let Some(error) = captured_error {
        return Err(error);
    }

    Ok(())
}
