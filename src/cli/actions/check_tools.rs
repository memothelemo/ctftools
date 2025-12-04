use anstream::{eprintln, println};
use anyhow::{Context, Result};
use console::Term;

use ctftools::registry::Toolkit;

use crate::ansi::{BOLD, GRAY, GREEN, RED};

pub fn run(term: &Term, toolkit: &Toolkit) -> Result<()> {
    term.hide_cursor()?;
    eprintln!("⏳ {BOLD}Checking the installation of all built-in tools...{BOLD:#}");

    let results = toolkit
        .check_install()
        .context("failed to check installation of all built-in tools")?;

    let total = results.len();
    term.show_cursor()?;
    crate::clear_last_lines(term, 1)?;

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
