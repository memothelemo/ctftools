use anyhow::Context;
use anyhow::{Result, bail};
use console::Term;
use log::debug;
use std::borrow::Cow;

use crate::cli::ansi::*;
use crate::env::Environment;
use crate::process::{ProcessBuilder, ProcessError};
use crate::registry::{ToolMetadata, ToolType};

pub fn run(env: &dyn Environment, stderr: &Term, tool: &ToolMetadata) -> Result<()> {
    if !env.is_live() {
        bail!("Mock environments are prohibited to run this action");
    }

    stderr.clear_screen()?;
    eprintln!("{BOLD}{} ({}){BOLD:#}", tool.name, tool.command);

    for line in wrap_text(&tool.description, stderr) {
        eprintln!("{GRAY}{line}{GRAY:#}");
    }

    match tool.kind {
        ToolType::Executable => run_as_executable(env, tool),
        ToolType::Website => run_as_link(tool),
    }?;
    eprintln!();

    Ok(())
}

fn run_as_link(tool: &ToolMetadata) -> Result<()> {
    eprintln!();
    eprint!(
        "{BOLD}Please enter for the tool selector to redirect \
        you to a link for you...{BOLD:#}",
    );

    let stdin = std::io::stdin();
    stdin.read_line(&mut String::new())?;

    let url = tool.url.as_ref().expect("url must be present in link tool");
    assert!(url.starts_with("https://") || url.starts_with("http://"));
    opener::open(url).context("failed to redirect to a link")?;

    Ok(())
}

fn run_as_executable(env: &dyn Environment, tool: &ToolMetadata) -> Result<()> {
    eprintln!();
    if !tool.examples.is_empty() {
        eprintln!("{BOLD}{GRAY}Examples:{GRAY:#}{BOLD:#}");
        for example in tool.examples.iter() {
            eprintln!("{GRAY}-{GRAY:#} {YELLOW}{example}{YELLOW:#}");
        }
        eprintln!();
    }

    eprintln!(
        "{BOLD}Please enter the arguments for {} to run \
        (press CTRL+C to abort):{BOLD:#}",
        tool.name
    );

    if let Ok(path) = std::env::current_dir() {
        eprintln!(
            "{GRAY}Your current directory is at: {}{GRAY:#}",
            path.display()
        );
    }

    eprintln!();
    eprint!("{} ", tool.command);

    let mut args = String::new();
    let stdin = std::io::stdin();

    match stdin.read_line(&mut args) {
        Ok(..) => {}
        Err(inner) if inner.kind() == std::io::ErrorKind::Interrupted => {
            debug!("got interrupted");
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };

    // Then we can create a brand new process to do this YAY
    let args = args.trim().to_string();
    let args = args.split(" ").collect::<Vec<_>>();
    let Some(cmd) = env.find_tool_executable(tool)? else {
        bail!(
            "I cannot run {} for you. Did you forget to install this tool?",
            tool.command
        )
    };

    eprintln!();

    let mut builder = ProcessBuilder::new(cmd);
    builder.args(&args);

    eprintln!("{GRAY}{builder}{GRAY:#}");
    let child = builder
        .build_command()
        .spawn()
        .with_context(|| ProcessError::could_not_execute(&builder))?;

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(ProcessError::new(
            &format!("process didn't exit successfully: {builder}"),
            Some(output.status),
            Some(&output),
        )
        .into());
    }

    Ok(())
}

#[must_use]
fn wrap_text<'s>(content: &'s str, term: &Term) -> Vec<Cow<'s, str>> {
    const PREFERRED_WIDTH: u16 = 100;

    let term_width = term.size().1;
    let wrapped_width = PREFERRED_WIDTH.min(term_width);
    textwrap::wrap(content, wrapped_width as usize)
}
