use anyhow::{Context, Result};
use console::Term;
use log::debug;

use ctftools::install::{InstallTask, InstallTaskError};
use ctftools::pkg::{AurHelper, PackageManager};
use ctftools::registry::{ToolMetadata, Toolkit};
use ctftools::util::pretty_cmd;

use crate::ansi::{BOLD, GRAY, YELLOW_BOLD};

pub fn run(term: &Term, toolkit: &Toolkit) -> Result<()> {
    // First, we need to find the missing built-in tools.
    let mut missing_tools = Vec::new();
    for (tool, installed) in toolkit.check_install()? {
        if !installed {
            missing_tools.push(tool);
        }
    }
    run_with_tools(term, missing_tools)
}

pub fn run_with_everything(term: &Term, toolkit: &Toolkit) -> Result<()> {
    run_with_tools(term, toolkit.tools().iter().collect::<Vec<_>>())
}

pub fn run_with_tools(term: &Term, tools_to_install: Vec<&ToolMetadata>) -> Result<()> {
    debug!(
        "found {} missing tool(s) to install",
        tools_to_install.len()
    );
    term.hide_cursor()?;

    let pkg_manager = PackageManager::detect()?;
    let aur_helper = AurHelper::detect()?;

    if pkg_manager.is_none() {
        eprintln!(
            "{YELLOW_BOLD}⚠️ It is recommened to install a package manager to automate \
        the process of installing the tools you need. Please install your \
        preferred package manager in your current operating system.{YELLOW_BOLD:#}"
        );
    }

    let mut tasks = Vec::new();
    for tool in tools_to_install {
        let command = tool.command.to_string();

        let mut task = None;
        if let Some((pkg_manager, path_to_pkgm)) = pkg_manager.clone() {
            let mut new_task =
                match InstallTask::from_package_manager(pkg_manager, path_to_pkgm, tool) {
                    Ok(okay) => okay,
                    Err(error @ InstallTaskError::PackageNotFound { .. }) => {
                        eprintln!("{YELLOW_BOLD}⚠️ {error}{YELLOW_BOLD:#}");
                        continue;
                    }
                    _ => unreachable!(),
                };

            if pkg_manager == PackageManager::Pacman
                && matches!(new_task, InstallTask::AUR { .. })
                && let Some((aur_helper, path_to_arh)) = aur_helper.clone()
            {
                new_task = match new_task {
                    InstallTask::AUR { package_name } => {
                        InstallTask::from_aur(aur_helper, path_to_arh, package_name)
                    }
                    fallback => fallback,
                };
            }

            task = Some(new_task);
        }

        if task.is_none() {
            task = match InstallTask::from_downloads(tool) {
                Ok(okay) => Some(okay),
                Err(error) => {
                    eprintln!("{YELLOW_BOLD}⚠️ {error}{YELLOW_BOLD:#}");
                    continue;
                }
            };
        }

        let task = task.expect("task should be Some");
        debug!("created install task for {command:?}; task = {task:?}");
        tasks.push(task);
    }

    // Log the missing tools so the user knows what's going with this command here
    eprintln!("⏳ {BOLD}Installing the following missing tools...{BOLD:#}");
    for task in tasks {
        match task {
            InstallTask::PackageManager {
                exec,
                arguments,
                sudo,
            } => {
                let mut cmd = ctftools::exec::make_cmd(exec, arguments, sudo);
                println!(
                    "{BOLD}Running{BOLD:#}: {GRAY}{}{GRAY:#}...",
                    pretty_cmd(&cmd)
                );
                ctftools::exec::run_cmd(&mut cmd, sudo).context("failed to install tool")?;
            }
            InstallTask::Download { .. } => todo!("Download link support"),
            InstallTask::AUR { .. } => todo!("AUR implementation"),
        }
    }

    Ok(())
}
