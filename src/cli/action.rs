use anyhow::{Context, Result};
use ctftools::registry::{ToolMetadata, Toolkit};
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use log::debug;
use std::borrow::Cow;

#[derive(Debug)]
pub enum Action<'a> {
    Tool(&'a ToolMetadata),
    InstallTools,
    InstallAllTools,
    CheckTools,
    Exit,
}

impl<'a> Action<'a> {
    /// Returns the display name of each corresponding action.
    #[must_use]
    pub fn display_name(&self) -> Cow<'static, str> {
        match self {
            Action::Tool(meta) => format!("🔨 {}", meta.name).into(),
            Action::CheckTools => "💻 Check tools".into(),
            Action::InstallAllTools => "💻 Install all tools (DEBUG)".into(),
            Action::InstallTools => "💻 Install missing tools".into(),
            Action::Exit => "🚪 Exit".into(),
        }
    }

    /// Generates a list of action that a user can allow to interact with the program.
    #[must_use]
    pub fn choices(toolkit: &'a Toolkit) -> Vec<Action<'a>> {
        let mut last_choices = vec![Action::CheckTools, Action::InstallTools];
        if crate::debug_enabled() {
            last_choices.push(Action::InstallAllTools);
        }
        last_choices.push(Action::Exit);

        let iter = toolkit.tools().iter().map(Action::Tool);
        iter.chain(last_choices).collect()
    }
}

pub fn prompt_select_action<'a>(toolkit: &'a Toolkit) -> Result<Option<Action<'a>>> {
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
