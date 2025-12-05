use anyhow::{Context, Result};
use console::Term;
use log::debug;

use ctftools::install::{InstallTask, InstallTaskError, check_toolkit_installation};
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
    install(term, missing_tools)
}

pub fn install_everything(term: &Term, toolkit: &Toolkit) -> Result<()> {
    install(term, toolkit.tools().iter().collect::<Vec<_>>())
}

pub fn install(term: &Term, tools_to_install: Vec<&ToolMetadata>) -> Result<()> {
    let tasks = crate::make_install_tasks::make_install_tasks(term, tools_to_install, true)?;

    // Log the missing tools so the user knows what's going with this command here
    eprintln!("⏳ {BOLD}Installing the following missing tools...{BOLD:#}");
    debug!("performing {} install task(s)", tasks.len());

    Ok(())
}
