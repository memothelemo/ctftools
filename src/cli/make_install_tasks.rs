use anyhow::Result;
use console::Term;
use log::debug;

use ctftools::install::{InstallTask, InstallTaskError};
use ctftools::pkg::{AurHelper, PackageManager};
use ctftools::registry::ToolMetadata;

use crate::ansi::YELLOW_BOLD;

pub fn make_install_tasks(
    term: &Term,
    tools_to_install: Vec<&ToolMetadata>,
    use_real_pm: bool,
) -> Result<Vec<InstallTask>> {
    debug!("got {} tool(s) to install", tools_to_install.len());
    term.hide_cursor()?;

    let pkg_manager = if use_real_pm {
        PackageManager::detect()?
    } else {
        None
    };

    let aur_helper = if use_real_pm {
        AurHelper::detect()?
    } else {
        None
    };

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

    Ok(tasks)
}

pub fn serialize_install_tasks(term: &Term, tools_to_install: Vec<&ToolMetadata>) -> Result<()> {
    let tasks = make_install_tasks(term, tools_to_install, false)?;
    let serialized = serde_json::to_string_pretty(&tasks)?;
    println!("{serialized}");

    Ok(())
}
