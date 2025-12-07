use crate::registry::{ToolMetadata, Toolkit};

use clap::Parser;
use std::borrow::Cow;

/// An action that can be performed in the interactive menu.
#[derive(Debug, PartialEq, Eq, Parser)]
pub enum Action<'a> {
    /// View details for a specific tool.
    #[clap(skip)]
    Tool(&'a ToolMetadata),

    /// Checks which tools in the toolkit are installed on the system.
    #[clap(name = "check")]
    CheckTools,

    /// Installs any tools from the toolkit that are not currently installed.
    #[cfg(feature = "auto-install-tools")]
    #[clap(name = "install")]
    InstallMissingTools,

    /// (Debug) Forcibly reinstalls all tools from the toolkit.
    #[cfg(all(debug_assertions, feature = "auto-install-tools"))]
    #[clap(name = "install-all")]
    InstallAllTools,

    /// Exits the application.
    #[clap(skip)]
    Exit,
}

impl<'a> Action<'a> {
    /// Returns the human-readable display name for each action.
    #[must_use]
    pub fn display_name(&self) -> Cow<'static, str> {
        match self {
            Action::Tool(meta) => format!("🔨 {}", meta.name).into(),
            Action::CheckTools => "🔎 Check which tools are installed".into(),
            #[cfg(feature = "auto-install-tools")]
            Action::InstallMissingTools => "📦 Install missing tools".into(),
            #[cfg(all(debug_assertions, feature = "auto-install-tools"))]
            Action::InstallAllTools => "🚀 Install all tools".into(),
            Action::Exit => "🚪 Exit".into(),
        }
    }

    /// Generates a list of available actions for the user to choose from.
    #[must_use]
    pub fn choices(toolkit: &'a Toolkit) -> Vec<Action<'a>> {
        let last = vec![Action::CheckTools];

        #[cfg(feature = "auto-install-tools")]
        last.push(Action::InstallMissingTools);

        let mut choices: Vec<Action<'a>> = toolkit
            .tools()
            .iter()
            .map(Action::Tool)
            .chain(last)
            .collect();

        #[cfg(all(debug_assertions, feature = "auto-install-tools"))]
        choices.push(Action::InstallAllTools);

        choices.push(Action::Exit);
        choices
    }
}
