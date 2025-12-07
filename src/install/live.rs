use anyhow::{Result, bail};
use cfg_if::cfg_if;
use std::time::Instant;

use crate::env::Environment;
use crate::install::{InstallProgress, InstallTask};
use crate::process::builder::LockedNotification;
use crate::process::{ProcessBuilder, ProcessError};

/// Innner implementation of [`run_install_task`] function in [`Environment`]
/// where the task must be [`InstallTask::PackageManager`] in order to perform
/// this function.
///
/// If the variant is different than expected, it will panic.
pub fn perform_task_via_pkg_manager(
    env: &dyn Environment,
    task: &InstallTask,
    progress_handler: &mut dyn FnMut(InstallProgress),
) -> Result<()> {
    let InstallTask::PackageManager {
        exec,
        arguments,
        sudo: needs_privilege,
        tool_name,
    } = task
    else {
        panic!("expected task to be InstallTask::PackageManager; got {task:?}")
    };

    // Check if this command requires elevated privileges.
    //
    // If so, verify whether the current process is running with sufficient privileges.
    //
    // If the process is not elevated and the OS does not support privilege escalation,
    // return an informative error message prompting the user to run with elevated privileges.
    if *needs_privilege && !env.running_in_elevation() && !env.supports_privilege_escalation() {
        cfg_if! {
            if #[cfg(target_os = "windows")] {
                bail!("Please run your terminal as administrator to allow memotools to install missing tools.");
            } else {
                bail!("Please run this command with elevated privileges to install missing tools.");
            }
        }
    }

    let mut builder = ProcessBuilder::new(exec);
    builder.args(arguments);

    if *needs_privilege && cfg!(unix) {
        builder.wrap(Some("sudo"));
    }

    let cmd_text = builder.to_string();
    let start_time = Instant::now();

    // Set up a flag that will be set to `true` when a `SIGINT` signal is received.
    progress_handler(InstallProgress::Command {
        text: cmd_text.clone(),
        tool_name: tool_name.clone(),
    });

    let output = builder.exec_locked(&mut |notification| match notification {
        LockedNotification::FirstWarning => {
            progress_handler(InstallProgress::InterruptFirstWarning);
        }
        LockedNotification::Interrupted => {
            progress_handler(InstallProgress::Interrupted);
        }
    })?;

    if !output.status.success() {
        return Err(ProcessError::new(
            &format!("process didn't exit successfully: {}", builder),
            Some(output.status),
            Some(&output),
        )
        .into());
    }

    // Report success.
    progress_handler(InstallProgress::Success {
        elapsed: start_time.elapsed(),
        tool_name: tool_name.clone(),
    });

    Ok(())
}
