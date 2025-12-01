use anstream::stream::IsTerminal;
use anstyle::{AnsiColor, Style};
use anyhow::{Context, Result, ensure};
use console::Term;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use memotools::registry::BUILTIN_TOOLS;

mod check_tools;
mod choices;
mod install_tools;

use self::choices::Choice;

fn print_header() {
    let gray = Style::new().fg_color(Some(AnsiColor::BrightBlack.into()));
    let header_gray = Style::new()
        .bold()
        .fg_color(Some(AnsiColor::BrightBlack.into()));

    let header = format!("CTF Tool Selector ({})", env!("CARGO_PKG_REPOSITORY"));
    eprintln!("{header_gray}{header}{header_gray:#}");
    eprintln!("{gray}Choose a tool to see quick usage notes.");

    eprint!("Press up or down arrow keys to select a choice");
    eprintln!("{gray:#}");

    eprintln!("{}", "-".repeat(header.len() + 2));
}

fn main() -> Result<()> {
    // Initialize tracing
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    // Load program's builtin tools upon running the program
    let _ = &*BUILTIN_TOOLS;

    // Prevent accidental termination via CTRL+C
    ctrlc::set_handler(|| {}).context("failed to set handler for CTRL+C")?;
    ensure!(std::io::stderr().is_terminal(), "stderr must be a terminal");

    // The main program code itself
    let mut term = Term::stderr();
    print_header();

    let choice = Choice::from_user_choice().unwrap();
    term.clear_last_lines(1)?;

    match choice {
        Choice::CheckTools => {
            self::check_tools::run(&mut term).context("failed to check for installation of tools")
        }
        Choice::InstallTools => {
            self::install_tools::run(&mut term).context("failed to install missing built-in tools")
        }
        Choice::Exit => {
            let green = Style::new()
                .fg_color(Some(AnsiColor::BrightGreen.into()))
                .bold();

            eprintln!("{green}Good luck to your CTFs!! 🚩🫶{green:#}");
            Ok(())
        }
        Choice::Tool(tool) => {
            let header = Style::new().bold();
            println!("{header}Selected tool{header:#}: {}", tool.name);
            println!("{header}Description{header:#}:\n{}", tool.description);

            Ok(())
        }
    }
}
