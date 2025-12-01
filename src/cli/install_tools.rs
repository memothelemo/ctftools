use anstyle::{AnsiColor, Color, Style};
use anyhow::{Context, Result, bail};
use console::Term;
use std::path::PathBuf;
use tracing::{debug, trace, warn};

use memotools::env::{AurHelper, PackageManager};
use memotools::registry::{BUILTIN_TOOLS, ToolMetadata};

static BOLD: Style = Style::new().bold();
static DIM: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));

#[derive(Debug)]
enum InstallTask {
    Executable {
        exec: PathBuf,
        args: Vec<String>,
        tool: ToolMetadata,
        requires_elevation: bool,
    },

    /// This task is applicable for Arch Linux.
    ManualArchInstall { tool: ToolMetadata },
}

impl InstallTask {
    #[must_use]
    pub fn tool(&self) -> &ToolMetadata {
        match self {
            Self::Executable { tool, .. } => tool,
            Self::ManualArchInstall { tool } => tool,
        }
    }
}

fn perform_install_task(term: &mut Term, task: InstallTask) -> Result<()> {
    match task {
        InstallTask::Executable {
            exec,
            args,
            requires_elevation,
            ..
        } => {
            // Check if we're running in elevation, otherwise throw the error
            // and let the user know that we need to elevate priliveges
            // to install the package.
            if !memotools::env::is_running_in_elevation() && requires_elevation {
                bail!("Please elevate prilivege to continue installing packages");
            }

            let output = memotools::process::run(exec, args)?;
            if output.status.success() {
                // Clear the entire screen
                term.clear_screen()?;
            } else {
                bail!("process failed");
            }

            Ok(())
        }
        InstallTask::ManualArchInstall { tool } => {
            todo!()
        }
    }
}

fn make_install_task(
    package_manager: &Option<(PackageManager, PathBuf)>,
    tool: ToolMetadata,
) -> Result<Option<InstallTask>> {
    // First thing we need to do is to whether it needs to
    // install with the AUR helper binary.
    //
    // Applicable if the preferred package manager is pacman.
    if matches!(package_manager, Some((PackageManager::Pacman, ..))) {
        let mut arch_package = tool.packages.get("pacman");
        let mut requires_aur = false;
        if arch_package.is_none() {
            arch_package = tool.packages.get("aur");
            requires_aur = arch_package.is_some();
        }

        // Or maybe in the defaults?
        if arch_package.is_none() {
            arch_package = tool.packages.get("default");
            requires_aur = false;
        }

        let Some(arch_package) = arch_package else {
            warn!(
                "I cannot find Arch equivalent package of {:?}. \
                Please install it manually.",
                tool.name
            );
            return Ok(None);
        };

        if requires_aur {
            let aur_helper = AurHelper::detect().context("cannot find preferred AUR helper")?;
            let task = if let Some((aur_helper, path)) = aur_helper {
                debug!("using AUR helper: {aur_helper:?}");
                InstallTask::Executable {
                    exec: path,
                    args: match aur_helper {
                        AurHelper::Paru => todo!(),
                        AurHelper::Yay => todo!(),
                    },
                    tool,
                    requires_elevation: aur_helper.requires_elevation(),
                }
            } else {
                InstallTask::ManualArchInstall { tool }
            };
            return Ok(Some(task));
        }

        let path = package_manager.as_ref().unwrap().1.to_path_buf();
        return Ok(Some(InstallTask::Executable {
            exec: path,
            args: vec!["-S".into(), "--noconfirm".into(), arch_package.into()],
            tool,
            requires_elevation: true,
        }));
    }

    // Second, if a package manager has installed in the operating
    // system, we'll use it other than to download a link or something.
    if let Some((package_manager, exec)) = package_manager.as_ref() {
        // Use the equivalent package from the preferred
        // package manager or use the default one?
        let package = tool
            .packages
            .get(package_manager.as_registry_key())
            .or_else(|| tool.packages.get("default"));

        let Some(package) = package else {
            warn!(
                "I cannot find {} equivalent package of {:?}. \
                Please install it manually.",
                package_manager.as_registry_key(),
                tool.name
            );
            return Ok(None);
        };

        // Voila! done!
        let args = match package_manager {
            PackageManager::Chocolatey => vec!["install".into(), package.to_string(), "-y".into()],
            PackageManager::WinGet => todo!(),
            PackageManager::Homebrew => todo!(),
            PackageManager::APT => todo!(),
            PackageManager::DNF => todo!(),
            PackageManager::Pacman => todo!(),
        };

        return Ok(Some(InstallTask::Executable {
            exec: exec.to_path_buf(),
            args,
            tool,
            requires_elevation: package_manager.requires_elevation(),
        }));
    }

    todo!()
}

pub fn run(term: &mut Term) -> Result<()> {
    term.hide_cursor()?;

    let package_manager = PackageManager::detect()?;
    if package_manager.is_none() {
        warn!(
            "It is recommened to install a package manager to automate \
        the process of installing the tools you need. Please install your \
        preferred package manager in your current operating system."
        );
    }

    let running_in_elevation = memotools::env::is_running_in_elevation();
    if let Some((value, ..)) = package_manager.as_ref() {
        debug!("using package manager: {value:?}");
    }
    debug!("running in elevation: {}", running_in_elevation);

    // First, we need to find the missing built-in tools.
    let mut missing_tools = BUILTIN_TOOLS.to_vec();
    // for tool in BUILTIN_TOOLS.iter() {
    //     if !memotools::tools::check_tool_install(tool)? {
    //         missing_tools.push(tool.clone());
    //     }
    // }

    // Second, use the missing built-in tools to make a
    // task command so we can identify issues with each
    // program as we go along the way.
    let mut tasks = Vec::new();
    for tool in missing_tools {
        let name = tool.name.to_string();
        trace!(?tool, "making install task for {name:?}");

        let Some(task) = make_install_task(&package_manager, tool)
            .with_context(|| format!("failed to create install task for {name:?}"))?
        else {
            continue;
        };

        trace!("successfully created task for installing {name:?} tool");
        tasks.push(task);
    }

    // Log the missing tools so the user knows what's going with this command here
    eprintln!("⏳ {BOLD}Installing the following missing tools...{BOLD:#}");
    for task in tasks.iter() {
        println!("- {DIM}{}{DIM:#}", task.tool().name);
    }

    // Third, we can finally run install tasks
    for task in tasks {
        let tool = task.tool();
        debug!(?task, "performing install task");

        let name = tool.name.clone();
        eprintln!("⏳ {BOLD}Installing {name}...{BOLD:#}");
        perform_install_task(term, task).with_context(|| format!("failed to install {name}"))?;
    }

    term.show_cursor()?;

    Ok(())
}
