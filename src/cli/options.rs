use clap::Parser;

#[derive(Debug, Parser)]
pub struct Options {
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// **Development option**
    ///
    /// This allows to plug a custom toolkit without using the
    /// default built-in toolkit that ships with ctftools program
    /// by inserting it with a JSON payload.
    #[cfg(debug_assertions)]
    #[clap(long)]
    pub custom_toolkit: Option<String>,
}

#[derive(Debug, Clone, Parser)]
pub enum Command {
    Check,
    InstallTools,

    /// **Development command**
    ///
    /// This allows to serialize the install tasks without using
    /// `ctftools install`, that will actually install all of the missing
    /// tools from the developer's environment.
    #[cfg(debug_assertions)]
    SerializeInstallTasks,
}
