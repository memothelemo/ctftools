use anstyle::{AnsiColor, Color, Style};
use anyhow::{Context, Result};

use std::path::PathBuf;
use std::process::{Output, Stdio};

static DIM: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));

pub fn run(exec: PathBuf, args: Vec<String>) -> Result<Output> {
    eprintln!("{DIM}{} {}{DIM:#}", exec.display(), args.join(" "));

    let output = std::process::Command::new(exec)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .context("failed to run process")?;

    Ok(output)
}
