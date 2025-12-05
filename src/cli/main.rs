use anstyle::Style;
use anyhow::{Context, Result};
use clap::Parser;
use console::Term;
use log::{debug, info};
use std::io::Write;

use ctftools::registry::Toolkit;

mod action;
mod ansi;
mod check_tools;
mod options;

use crate::action::Action;
use crate::ansi::{BOLD, GRAY};
use crate::options::{Command, Options};

fn main() -> Result<()> {
    // Initialize logger
    init_logger();

    // Parsing the program's starting arguments into CLI options
    let opts = Options::parse();

    // Prevent accidental termination via CTRL+C (unless we need to)
    ctrlc::set_handler(|| {}).context("failed to set handler for CTRL+C")?;

    // Load our toolkit to be used for the entire program's lifetime.
    let toolkit = init_maybe_custom_toolkit(&opts)?;
    let term = Term::stderr();

    let mut action = match opts.command {
        Some(Command::Check) => Some(Action::CheckTools),
        Some(Command::InstallTools) => Some(Action::InstallTools),
        None => None,
    };

    loop {
        print_header();

        let mut interactive_mode = true;
        let action = match action.take() {
            // This came from the CLI so yes...
            Some(action) => {
                print_header_line();
                interactive_mode = false;
                Some(action)
            }
            None => {
                print_header_prompt_instructions();
                print_header_line();
                self::action::prompt_select_action(&toolkit)?
            }
        };
        term.show_cursor()?; // try to restore our cursor if CTRL+C has triggered

        // Clear the usage and the extra line so we can try to perform an action?
        if action.is_some() && interactive_mode {
            clear_last_lines(&term, 4)?;
        }

        // If the prompt is interrupted, we can assume that the user
        // wants to exit the program.
        let action = action.unwrap_or(Action::Exit);
        debug!("selected action: {action:?}");

        let result = match action {
            Action::Tool(metadata) => todo!(),
            Action::InstallTools => todo!(),
            Action::InstallAllTools => todo!(),
            Action::CheckTools => self::check_tools::run(&term, &toolkit),
            Action::Exit => break,
        };
        term.show_cursor()?;

        // If it has an error then, immediately stop the program.
        if let Err(error) = result {
            return Err(error).context("error occurred while trying to perform this command");
        }

        // Then prompt the user if they want to go back to the menu
        if !interactive_mode || !prompt_repeat()? {
            break;
        }

        eprintln!();
    }

    Ok(())
}

fn clear_last_lines(term: &Term, n: usize) -> std::io::Result<()> {
    // Do not clear last lines if debug logging is enabled.
    if debug_enabled() {
        Ok(())
    } else {
        term.clear_last_lines(n)
    }
}

const CLI_HEADER: &str = concat!("CTF Tool Selector (", env!("CARGO_PKG_REPOSITORY"), ")");

fn prompt_repeat() -> Result<bool> {
    fn prompt_repeat_inner() -> Result<Option<bool>> {
        let input: Option<String> = dialoguer::Input::new()
            .with_prompt("Do you want to select a tool again? [Y/n]")
            .interact_text()
            .map(Some)
            .or_else(|error| match error {
                dialoguer::Error::IO(inner) if inner.kind() == std::io::ErrorKind::Interrupted => {
                    Ok(None)
                }
                dialoguer::Error::IO(error) => Err(error),
            })
            .context("failed to prompt response")?;

        // If it's interrupted, then force exit!
        let Some(response) = input else {
            eprintln!();
            debug!("got interrupted");
            return Ok(Some(false));
        };

        match response.chars().next().map(|v| v.to_ascii_lowercase()) {
            Some('y') => Ok(Some(true)),
            Some('n') => Ok(Some(false)),
            _ => Ok(None),
        }
    }

    loop {
        match prompt_repeat_inner()? {
            Some(true) => return Ok(true),
            Some(false) => return Ok(false),
            None => {}
        };
    }
}

fn print_header() {
    eprintln!("{BOLD}{CLI_HEADER}{BOLD:#}");
}

fn print_header_line() {
    eprintln!("{}", "-".repeat(CLI_HEADER.len() + 2));
}

fn print_header_prompt_instructions() {
    eprintln!("{GRAY}Choose a tool to see quick usage notes.");

    eprint!("Press up or down arrow keys to select a choice");
    eprintln!("{GRAY:#}");
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
