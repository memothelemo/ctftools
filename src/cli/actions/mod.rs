use memotools::registry::{ToolMetadata, Toolkit};
use std::borrow::Cow;

pub mod check_tools;
pub mod install_tools;

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
