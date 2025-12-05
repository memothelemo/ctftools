use anstyle::Style;
use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info};
use std::io::Write;

use ctftools::registry::Toolkit;

mod ansi;
mod check_tools;
mod options;

use crate::ansi::{BOLD, GRAY};
use crate::options::{Command, Options};

fn main() -> Result<()> {
    // Initialize logger
    init_logger();

    // Parsing the program's starting arguments into CLI options
    let opts = Options::parse();

    // Prevent accidental termination via CTRL+C (unless we need to)
    ctrlc::set_handler(|| {}).context("failed to set handler for CTRL+C")?;
    print_header();

    // Load our toolkit to be used for the entire program's lifetime.
    let toolkit = init_maybe_custom_toolkit(&opts)?;
    match opts.command {
        Some(Command::Check) => {
            self::check_tools::run(&toolkit)?;
        }
        Some(Command::InstallTools) => todo!(),
        None => todo!(),
    }

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

fn init_maybe_custom_toolkit(opts: &Options) -> Result<Toolkit> {
    if let Some(json) = opts.custom_toolkit.as_ref() {
        let toolkit = Toolkit::from_json(json).context("could not load custom toolkit")?;
        debug!(
            "using custom toolkit; loaded tool(s) = {}",
            toolkit.tools().len()
        );
        Ok(toolkit)
    } else {
        Ok(Toolkit::default().clone())
    }
}

fn debug_enabled() -> bool {
    std::env::var("CTFTOOLS_DEBUG").as_deref().unwrap_or("0") != "0"
}

fn init_logger() {
    use self::ansi::*;

    let debug_enabled = debug_enabled();
    let _ = env_logger::Builder::new()
        .filter_level(if debug_enabled {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Warn
        })
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

            let mut module_path = record.module_path().unwrap_or("unknown");

            // If the module path is memotools, replace it with `memotools::cli`
            // since we haven't added any functions in the main library module.
            if module_path == "memotools" {
                module_path = "memotools::cli";
            }

            write!(
                buf,
                "{GRAY}] {module_path}:{}{GRAY:#} - ",
                record.line().unwrap_or_default()
            )?;

            writeln!(buf, "{}", record.args())
        })
        .format_timestamp(None)
        .try_init();

    if debug_enabled {
        info!("Debug logging is enabled");
    }
}
