use anyhow::{Context, Result};
use console::{Color, Style};
use dialoguer::{Select, theme::ColorfulTheme};
use memotools::registry::{BUILTIN_TOOLS, ToolMetadata};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    Tool(ToolMetadata),
    CheckTools,
    InstallTools,
    Exit,
}

impl Choice {
    /// Returns the display name of each corresponding choice.
    #[must_use]
    pub fn display_name(&self) -> Cow<'static, str> {
        match self {
            Choice::Tool(meta) => format!("🔨 {}", meta.name).into(),
            Choice::CheckTools => "⚙️ Check tools".into(),
            Choice::InstallTools => "⚙️ Install missing tools".into(),
            Choice::Exit => "🚪 Exit".into(),
        }
    }

    #[must_use]
    pub fn from_user_choice() -> Result<Self> {
        let choices = Self::choices();
        let names = choices.iter().map(Choice::display_name).collect::<Vec<_>>();

        let theme = ColorfulTheme {
            active_item_style: Style::new().bold().fg(Color::Green),
            ..Default::default()
        };

        let idx = Select::with_theme(&theme)
            .default(0)
            .items(names)
            .report(true)
            .interact()
            .or_else(|error| match error {
                dialoguer::Error::IO(inner) if inner.kind() == std::io::ErrorKind::Interrupted => {
                    // our sentiel value for the last index of the choices (which is Exit choice)
                    Ok(choices.len() - 1)
                }
                dialoguer::Error::IO(error) => Err(error),
            })
            .context("failed to prompt choice")?;

        let choice = choices
            .into_iter()
            .nth(idx)
            .expect("should return a valid choice corresponding to its index");

        Ok(choice)
    }

    /// Generates a list of choices that a user can allow
    /// to interact with the program.
    pub fn choices() -> Vec<Choice> {
        let last_choices = [Choice::CheckTools, Choice::InstallTools, Choice::Exit];
        BUILTIN_TOOLS
            .iter()
            .cloned()
            .map(Choice::Tool)
            .chain(last_choices)
            .collect()
    }
}
