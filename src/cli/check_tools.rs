use anstream::{eprintln, println};
use anstyle::{AnsiColor, Color, Style};
use anyhow::{Context, Result};
use console::Term;
use memotools::registry::BUILTIN_TOOLS;

static BOLD: Style = Style::new().bold();
static DIM: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));

static GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
static RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

pub fn run(term: &mut Term) -> Result<()> {
    term.hide_cursor()?;
    eprintln!("⏳ {BOLD}Checking the installation of all built-in tools...{BOLD:#}");

    let results = BUILTIN_TOOLS
        .iter()
        .map(|tool| {
            let installed = memotools::tools::check_tool_install(tool)?;
            Ok::<_, _>((tool, installed))
        })
        .collect::<Result<Vec<_>>>()
        .context("failed to check installation of all built-in tools")?;

    let total = results.len();
    term.show_cursor()?;
    term.clear_last_lines(1)?;

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
            "{DIM}{BOLD}You may want to return the selector again to install \
            the missing tools.{BOLD:#}{DIM:#}"
        );
    }

    Ok(())
}
