use anyhow::{Context, Result, anyhow, bail};
use cfg_if::cfg_if;
use log::debug;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Instant;
use tempdir::TempDir;
use tokio::io::AsyncWriteExt;

use crate::env::Environment;
use crate::install::{InstallProgress, InstallTask};
use crate::process::builder::LockedNotification;
use crate::process::{ProcessBuilder, ProcessError};
use crate::registry::DownloadFileFormat;

/// Inner implementation of [`run_install_task`] function in [`Environment`]
/// where the task must be [`InstallTask::Download`] in order to perform
/// this function.
///
/// If the variant is different than expected, it will panic.
pub fn perform_task_via_download(
    _env: &dyn Environment,
    task: &InstallTask,
    _progress_handler: &mut dyn FnMut(InstallProgress),
) -> Result<()> {
    let InstallTask::Download {
        instructions,
        tool_name,
    } = task
    else {
        panic!("expected task to be InstallTask::Download; got {task:?}")
    };

    // First, we'll add a temporary folder to capture the installer executables.
    let dir = TempDir::new("ctftools_download")?;
    let downloaded_path = dir.path().join(match instructions.format {
        DownloadFileFormat::Executable => {
            if cfg!(windows) {
                "downloaded.exe"
            } else {
                "downloaded.zip"
            }
        }
        DownloadFileFormat::ZIP => "downloaded.zip",
    });

    // Unfortunately, this part requires a bit of an async action but we have
    // our channel to send progress messages in the async thread.
    let (tx, _rx) = mpsc::channel::<InstallProgress>();
    let tool_name = tool_name.clone();
    let url = instructions.url.clone();

    let handle = std::thread::spawn({
        let downloaded_path = downloaded_path.clone();
        move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .thread_name(format!("ctftools-download-worker-{tool_name}"))
                .enable_all()
                .worker_threads(1)
                .build()
                .expect("failed to build tokio runtime for download worker");

            rt.block_on(download_file_from_url(&tx, downloaded_path, url))
        }
    });

    handle
        .join()
        .map_err(|_| anyhow!("failed to spawn download worker"))?
        .context("failed to download file")?;

    // Once the download is complete, let's open the file. shall we?
    match instructions.format {
        DownloadFileFormat::Executable => {
            try_open_executable(&downloaded_path)?;
        }
        DownloadFileFormat::ZIP => todo!(),
    }

    dir.close()?;
    Ok(())
}

fn try_open_executable(path: &Path) -> Result<()> {
    let mut builder = ProcessBuilder::new(path);
    if cfg!(windows) {
        builder.wrap(Some("start"));
    }

    debug!("executing: {builder}");
    builder.exec_with_output()?;
    Ok(())
}

async fn download_file_from_url(
    _progress_tx: &mpsc::Sender<InstallProgress>,
    path: PathBuf,
    url: String,
) -> Result<()> {
    debug!("fetching resource: {url}");

    let mut response = reqwest::get(url).await.context("HTTP request failed")?;
    let mut file = tokio::fs::File::create(&path)
        .await
        .context("could not create a temporary downloaded file")?;

    debug!("created temporary file: {}", path.display());

    let mut bytes_written = 0usize;
    let total_bytes = response.content_length().map(|v| v as usize);

    while let Some(bytes) = response.chunk().await? {
        if let Some(total_bytes) = total_bytes {
            debug!("received {bytes_written}/{total_bytes} byte(s) from stream",);
        } else {
            debug!("received {bytes_written} byte(s) from stream",);
        }
        bytes_written += bytes.len();
        file.write(&bytes).await?;
    }

    debug!("downloaded {bytes_written} byte(s)");
    file.flush().await?;

    Ok(())
}

/// Inner implementation of [`run_install_task`] function in [`Environment`]
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
