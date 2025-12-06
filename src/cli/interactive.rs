use anyhow::{Context, Result};
use console::Term;
use console::{Color, Style};
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use log::debug;

use crate::cli::ansi::*;
use crate::cli::{Action, TermExt};
use crate::env::Environment;
use crate::registry::Toolkit;

pub fn enter_interactive_mode(
    env: &dyn Environment,
    stderr: &Term,
    toolkit: &Toolkit,
) -> Result<()> {
    debug!("entering interactive mode");
    loop {
        print_cli_header();
        print_cli_header_line();
        print_select_action_instructions();

        // try to restore our cursor if CTRL+C has triggered
        let action = prompt_select_action(toolkit)?;
        stderr.show_cursor()?;

        // clear the usage and the extra line so we can try to perform an action?
        if action.is_some() {
            stderr.clear_lines(3)?;
        }

        // If the prompt is interrupted, we can assume that the user
        // wants to exit the program.
        let action = action.unwrap_or(Action::Exit);
        debug!("selected action: {action:?}");

        // Immediately stop the loop if the user selected exit action
        if let Action::Exit = action {
            break;
        }

        let result = crate::cli::try_run_action(action, env, stderr, toolkit);
        stderr.show_cursor()?;

        // If it has an error then, immediately stop the program.
        if let Err(error) = result {
            return Err(error).context("error occurred while trying to perform this command");
        }

        // Then prompt the user if they want to go back to the interactive menu
        if !prompt_yes_or_no("Do you want to select a tool again?")?.unwrap_or(true) {
            break;
        }
    }

    print_goodbye_message();
    Ok(())
}

pub fn prompt_yes_or_no(question: &str) -> Result<Option<bool>> {
    loop {
        let input: Option<String> = dialoguer::Input::new()
            .with_prompt(format!("{question} [Y/n]"))
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
        let Some(input) = input else {
            debug!("got interrupted");
            return Ok(None);
        };

        let response = input.chars().next().map(|v| v.to_ascii_lowercase());
        match response {
            Some('y') => return Ok(Some(true)),
            Some('n') => return Ok(Some(false)),
            _ => {}
        }
    }
}

/// Prompts the user to select an action from a list of choices.
///
/// This function displays an interactive, fuzzy-searchable menu.
///
/// # Returns
/// - `Ok(Some(Action))` if the user selects an action.
/// - `Ok(None)` if the user cancels the prompt (e.g., by pressing Esc or Ctrl+C).
/// - `Err` if an I/O error occurs.
pub fn prompt_select_action<'a>(toolkit: &'a Toolkit) -> Result<Option<Action<'a>>> {
    let choices = Action::choices(toolkit);
    let theme = ColorfulTheme {
        active_item_style: Style::new().bold().fg(Color::Green),
        ..Default::default()
    };

    // Get the list of display names for the prompt.
    let items: Vec<_> = choices.iter().map(Action::display_name).collect();

    let idx = FuzzySelect::with_theme(&theme)
        .default(0)
        .items(&items)
        .report(false)
        .interact()
        .map(Some)
        .or_else(|error| match error {
            dialoguer::Error::IO(inner) if inner.kind() == std::io::ErrorKind::Interrupted => {
                debug!("got interrupted");
                Ok(None)
            }
            dialoguer::Error::IO(error) => Err(error),
        })
        .context("failed to prompt choice")?;

    if let Some(idx) = idx {
        Ok(choices.into_iter().nth(idx))
    } else {
        Ok(None)
    }
}

const CLI_HEADER: &str = concat!("CTF Tool Selector (", env!("CARGO_PKG_REPOSITORY"), ")");

pub fn print_cli_header() {
    eprintln!("{BOLD}{CLI_HEADER}{BOLD:#}");
}

pub fn print_cli_header_line() {
    eprintln!("{}", "-".repeat(CLI_HEADER.len() + 2));
}

pub fn print_goodbye_message() {
    eprintln!("{GREEN_BOLD}Good luck to your CTFs!! 🚩🫶{GREEN_BOLD:#}");
}

fn print_select_action_instructions() {
    eprintln!("{GRAY}Choose a tool to see quick usage notes.");

    eprint!("Press up or down arrow keys to select a choice");
    eprintln!("{GRAY:#}");
}
