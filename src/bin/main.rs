use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use ctftools::cli;
use ctftools::cli::ansi::YELLOW_BOLD;
use ctftools::env::{Environment, LiveEnvironment, MockEnvironment};

fn main() -> Result<()> {
    // Parsing the program's starting arguments into CLI options
    let mut opts = cli::Options::parse();

    // Do we need to go to live or mock?
    let env: Arc<dyn Environment> = if let Some(tools) = opts.mock_installed_tools.take() {
        // Warn the developer that they are using a mocked environment.
        eprintln!(
            "{YELLOW_BOLD}⚠️ WARNING: You are running ctftools with a mocked system environment. \
            This feature is intended for automated testing and may result unexpected behavior.\
            {YELLOW_BOLD:#}"
        );
        eprintln!();

        let mut map = HashMap::new();
        tools.into_iter().for_each(|name| {
            map.insert(name, PathBuf::new());
        });

        Arc::new(MockEnvironment::builder().installed_tools(map).build())
    } else {
        Arc::new(LiveEnvironment::new()?)
    };

    ctftools::cli::run(env, opts, None)
}
