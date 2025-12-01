use bon::Builder;
use clap::Parser;

use crate::cli::action::Action;

#[derive(Debug, Builder, Parser)]
pub struct Options {
    #[clap(subcommand)]
    pub action: Option<Action<'static>>,

    /// **Development option**
    ///
    /// This allows to plug a custom toolkit without using the
    /// default built-in toolkit that ships with ctftools program
    /// by inserting it with a JSON payload.
    #[cfg(debug_assertions)]
    #[clap(long)]
    pub custom_toolkit: Option<String>,

    /// **Development option**
    ///
    /// Mocks the presence of certain tools for testing purposes.
    /// Use a comma-separated list to specify tool names.
    #[cfg(debug_assertions)]
    #[clap(long, value_delimiter = ',')]
    pub mock_installed_tools: Option<Vec<String>>,
}
