use anstream::stream::IsTerminal;
use anstyle::{AnsiColor, Style};
use anyhow::{Context, Result, ensure};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use memotools::registry::BUILTIN_TOOLS;

fn print_header() {
    let gray = Style::new().fg_color(Some(AnsiColor::BrightBlack.into()));
    let header = format!("CTF Tool Selector ({})", env!("CARGO_PKG_REPOSITORY"));
    eprintln!("{gray}{header}");
    eprintln!("Choose a tool to see quick usage notes.{gray:#}");
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
    print_header();

    Ok(())
}
