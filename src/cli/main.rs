use anstream::stream::IsTerminal;
use anstyle::Style;
use anyhow::{Context, Result, ensure};
use console::Term;
use dialoguer::FuzzySelect;
use dialoguer::theme::ColorfulTheme;
use log::{debug, info};
use std::io::Write;

use memotools::registry::Toolkit;

mod actions;
use crate::actions::Action;

/// Collection of ANSI colors for quick convenience.
mod ansi;
use crate::ansi::{BOLD, GRAY, GREEN_BOLD};

fn main() -> Result<()> {
    // Initialize logger
    init_logger();

    // Load program's builtin tools upon running the program
    let toolkit = Toolkit::default();

    // Prevent accidental termination via CTRL+C (unless we need to)
    ctrlc::set_handler(|| {}).context("failed to set handler for CTRL+C")?;
    ensure!(std::io::stderr().is_terminal(), "stderr must be a terminal");

    // The main program code itself
    let term = Term::stderr();
    'main: loop {
        print_header();

        let action = prompt_select_action(toolkit)?;
        term.show_cursor()?; // try to restore our cursor if CTRL+C has triggered

        // Clear the usage and the extra line so we can try to perform an action?
        if action.is_some() {
            clear_last_lines(&term, 4)?;
        }

        // If the prompt is interrupted, we can assume that the user
        // wants to exit the program.
        let action = action.unwrap_or(Action::Exit);
        debug!("selected action: {action:?}");

        let result = match action {
            Action::Tool(metadata) => todo!(),
            Action::InstallTools => self::actions::install_tools::run(&term, toolkit),
            Action::InstallAllTools => {
                self::actions::install_tools::run_with_everything(&term, toolkit)
            }
            Action::CheckTools => self::actions::check_tools::run(&term, toolkit),
            Action::Exit => break,
        };
        term.show_cursor()?;

        // If it has an error then, immediately stop the program.
        if let Err(error) = result {
            return Err(error).context("error occurred while trying to perform this command");
        }

        // Then prompt the user if they want to go back to the menu
        loop {
            match prompt_repeat()? {
                Some(true) => break,
                Some(false) => break 'main,
                None => {}
            };
        }
        eprintln!();
    }

    print_exit_message();
    Ok(())
}

fn prompt_repeat() -> Result<Option<bool>> {
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

fn prompt_select_action<'a>(toolkit: &'a Toolkit) -> Result<Option<Action<'a>>> {
    use console::{Color, Style};

    let choices = Action::choices(toolkit);
    let theme = ColorfulTheme {
        active_item_style: Style::new().bold().fg(Color::Green),
        ..Default::default()
    };

    let idx = FuzzySelect::with_theme(&theme)
        .default(0)
        .items(choices.iter().map(Action::display_name).collect::<Vec<_>>())
        .report(false)
        .interact()
        .or_else(|error| match error {
            dialoguer::Error::IO(inner) if inner.kind() == std::io::ErrorKind::Interrupted => {
                debug!("got interrupted");
                Ok(usize::MAX)
            }
            dialoguer::Error::IO(error) => Err(error),
        })
        .context("failed to prompt choice")?;

    let choice = choices.into_iter().nth(idx);
    Ok(choice)
}

fn print_header() {
    let header = format!("CTF Tool Selector ({})", env!("CARGO_PKG_REPOSITORY"));
    eprintln!("{BOLD}{header}{BOLD:#}");
    eprintln!("{GRAY}Choose a tool to see quick usage notes.");

    eprint!("Press up or down arrow keys to select a choice");
    eprintln!("{GRAY:#}");

    eprintln!("{}", "-".repeat(header.len() + 2));
}

fn clear_last_lines(term: &Term, n: usize) -> std::io::Result<()> {
    // Do not clear last lines if debug logging is enabled.
    if debug_enabled() {
        Ok(())
    } else {
        term.clear_last_lines(n)
    }
}

fn print_exit_message() {
    eprintln!("{GREEN_BOLD}Good luck to your CTFs!! 🚩🫶{GREEN_BOLD:#}");
}

fn debug_enabled() -> bool {
    std::env::var("CTFTOOLS_DEBUG").as_deref().unwrap_or("0") != "0"
}

fn init_logger() {
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
