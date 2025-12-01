use anstream::stream::IsTerminal;
use anstyle::{AnsiColor, Style};
use anyhow::{Context, Result, ensure};

fn print_header() {
    let gray = Style::new().fg_color(Some(AnsiColor::BrightBlack.into()));
    let header = format!("CTF Tool Selector ({})", env!("CARGO_PKG_REPOSITORY"));
    eprintln!("{gray}{header}");
    eprintln!("Choose a tool to see quick usage notes.{gray:#}");
    eprintln!("{}", "-".repeat(header.len() + 2));
}

fn main() -> Result<()> {
    // Prevent accidental termination via CTRL+C
    ctrlc::set_handler(|| {}).context("failed to set handler for CTRL+C")?;
    ensure!(std::io::stderr().is_terminal(), "stderr must be a terminal");
    print_header();

    Ok(())
}
