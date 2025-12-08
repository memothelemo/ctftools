use anyhow::Context;
use anyhow::{Result, bail};
use console::Term;
use log::debug;
use std::borrow::Cow;

use crate::cli::ansi::*;
use crate::env::Environment;
use crate::process::{ProcessBuilder, ProcessError};
use crate::registry::ToolMetadata;

pub fn run(env: &dyn Environment, stderr: &Term, tool: &ToolMetadata) -> Result<()> {
    if !env.is_live() {
        bail!("Mock environments are prohibited to run this action");
    }

    stderr.clear_screen()?;
    eprintln!("{BOLD}{} ({}){BOLD:#}", tool.name, tool.command);

    for line in wrap_text(&tool.description, stderr) {
        eprintln!("{GRAY}{line}{GRAY:#}");
    }

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

    eprintln!();
    Ok(())
}

#[must_use]
fn wrap_text<'s>(content: &'s str, term: &Term) -> Vec<Cow<'s, str>> {
    const PREFERRED_WIDTH: u16 = 100;

    let term_width = term.size().1;
    let wrapped_width = PREFERRED_WIDTH.min(term_width);
    textwrap::wrap(content, wrapped_width as usize)
}
