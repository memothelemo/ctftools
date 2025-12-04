use anstream::stream::IsTerminal;
use anstyle::{AnsiColor, Style};
use anyhow::{Context, Result, ensure};
use console::Term;
use std::io::Write;

use memotools::registry::Toolkit;
use memotools::util::ansi::{BOLD, GRAY};

fn main() -> Result<()> {
    // Initialize debug logger if enabled
    if debug_enabled() {
        init_debug_logger();
    }

    // Load program's builtin tools upon running the program
    let _ = Toolkit::default();

    ctrlc::set_handler(|| {}).context("failed to set handler for CTRL+C")?;
    ensure!(std::io::stderr().is_terminal(), "stderr must be a terminal");

    // The main program code itself
    // let mut term = Term::stderr();
    print_header();

    Ok(())
}

fn print_header() {
    let header = format!("CTF Tool Selector ({})", env!("CARGO_PKG_REPOSITORY"));
    eprintln!("{BOLD}{header}{BOLD:#}");
    eprintln!("{GRAY}Choose a tool to see quick usage notes.");

    eprint!("Press up or down arrow keys to select a choice");
    eprintln!("{GRAY:#}");

    eprintln!("{}", "-".repeat(header.len() + 2));
}

fn debug_enabled() -> bool {
    std::env::var("CTFTOOLS_DEBUG").as_deref().unwrap_or("0") != "0"
}

fn init_debug_logger() {
    let _ = env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .format(|buf, record| {
            use anstyle::AnsiColor;
            use log::Level;

            write!(buf, "{GRAY}[{GRAY:#}")?;

            let (level_str, color) = match record.level() {
                Level::Error => ("ERROR ", AnsiColor::BrightRed),
                Level::Warn => ("WARN ", AnsiColor::BrightYellow),
                Level::Info => ("INFO ", AnsiColor::BrightGreen),
                Level::Debug => ("DEBUG", AnsiColor::BrightBlue),
                Level::Trace => ("TRACE", AnsiColor::BrightMagenta),
            };
            let style = Style::new().fg_color(Some(color.into()));
            write!(buf, "{style}{level_str}{style:#}")?;

            write!(
                buf,
                "{GRAY}] {}:{}{GRAY:#} - ",
                record.module_path().unwrap_or("unknown"),
                record.line().unwrap_or_default()
            )?;

            writeln!(buf, "{}", record.args())
        })
        .format_timestamp(None)
        .try_init();
}
