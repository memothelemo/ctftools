use anstream::{eprintln, println};
use anyhow::{Context, Result};
use console::Term;

use crate::cli::TermExt;
use crate::cli::ansi::*;
use crate::env::Environment;
use crate::registry::Toolkit;

pub fn run(env: &dyn Environment, stderr: &Term, toolkit: &Toolkit) -> Result<()> {
    stderr.hide_cursor()?;
    eprintln!("⏳ {BOLD}Checking the installation of all built-in tools...{BOLD:#}");

    let results = env
        .check_toolkit_installation(toolkit)
        .context("failed to check installation of all built-in tools")?;

    let total = results.len();
    stderr.show_cursor()?;
    stderr.clear_lines(1)?;

    let divider = "=".repeat(25);
    eprintln!("{BOLD}{divider} Built-in Tools {divider}{BOLD:#}");

    let mut installed_count = 0usize;
    for (tool, installed) in results {
        let (emoji, style) = if installed {
            installed_count += 1;
            ('✅', GREEN)
        } else {
            ('❌', RED)
        };
        println!("* {style}{emoji} {}{style:#}", tool.name);
    }

    eprintln!();
    if installed_count == total {
        println!(
            "{GREEN}{BOLD}All done! {installed_count}/{total} tools installed.{BOLD:#}{GREEN:#}"
        );
    } else {
        let missing = total - installed_count;
        println!("{RED}{BOLD}Missing tools: {missing}/{total}{BOLD:#}{RED:#}");
        println!(
            "{GRAY}{BOLD}You may want to return the selector again to install \
            the missing tools.{BOLD:#}{GRAY:#}"
        );
    }

    Ok(())
}
