use anstream::{eprintln, println};
use anyhow::Result;

use console::Term;
use ctftools::install::check_toolkit_installation;
use ctftools::registry::Toolkit;

use crate::ansi::*;

pub fn run(term: &Term, toolkit: &Toolkit) -> Result<()> {
    eprintln!("⏳ {BOLD}Checking the installation of all built-in tools...{BOLD:#}");

    let results = check_toolkit_installation(toolkit)?;
    crate::clear_last_lines(term, 1)?;

    let mut installed_count = 0usize;
    let expected_count = results.len();

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
    if installed_count == expected_count {
        println!(
            "{GREEN}{BOLD}All done! {installed_count}/{expected_count} tools installed.{BOLD:#}{GREEN:#}"
        );
    } else {
        let missing = expected_count - installed_count;
        println!("{RED}{BOLD}Missing tools: {missing}/{expected_count}{BOLD:#}{RED:#}");
        println!(
            "{GRAY}{BOLD}You may want to return the selector again to install \
            the missing tools.{BOLD:#}{GRAY:#}"
        );
    }

    Ok(())
}
