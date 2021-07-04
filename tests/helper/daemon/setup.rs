use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::{bail, Context, Result};
use assert_cmd::prelude::*;
use tempdir::TempDir;
use tokio::io::{self, AsyncWriteExt};

use pueue_daemon_lib::run;
use pueue_lib::settings::*;

use super::{get_pid, sleep_ms};

/// Spawn the daemon main logic in it's own async function.
/// It'll be executed by the tokio multi-threaded executor.
pub fn boot_daemon(pueue_dir: &Path) -> Result<i32> {
    let path = pueue_dir.clone().to_path_buf();
    // Start/spin off the daemon and get its PID
    tokio::spawn(run_and_handle_error(path, true));
    let pid = get_pid(pueue_dir)?;

    let tries = 20;
    let mut current_try = 0;

    // Wait up to 1s for the unix socket to pop up.
    let socket_path = pueue_dir.join("pueue.pid");
    while current_try < tries {
        sleep_ms(50);
        if socket_path.exists() {
            return Ok(pid);
        }

        current_try += 1;
    }

    bail!("Daemon didn't boot after 1sec")
}

/// Spawn the daemon by calling the actual pueued binary.
/// This function also checks for the pid file and the unix socket to pop-up.
pub fn boot_standalone_daemon(pueue_dir: &Path) -> Result<Child> {
    let child = Command::cargo_bin("pueued")?
        .arg("--config")
        .arg(pueue_dir.join("pueue.yml").to_str().unwrap())
        .arg("-vvv")
        .stdout(Stdio::piped())
        .spawn()?;

    let tries = 20;
    let mut current_try = 0;

    // Wait up to 1s for the unix socket to pop up.
    let socket_path = pueue_dir.join("pueue.pid");
    while current_try < tries {
        sleep_ms(50);
        if socket_path.exists() {
            return Ok(child);
        }

        current_try += 1;
    }

    bail!("Daemon didn't boot in stand-alone mode after 1sec")
}

/// Internal helper function, which wraps the daemon main logic and prints any errors.
async fn run_and_handle_error(pueue_dir: PathBuf, test: bool) -> Result<()> {
    if let Err(err) = run(Some(pueue_dir.join("pueue.yml")), test).await {
        let mut stdout = io::stdout();
        stdout
            .write_all(format!("Entcountered error: {:?}", err).as_bytes())
            .await
            .expect("Failed to write to stdout.");
        stdout.flush().await?;

        return Err(err);
    }

    Ok(())
}

pub fn base_setup() -> Result<(Settings, TempDir)> {
    // Create a temporary directory used for testing.
    let tempdir = TempDir::new("pueue_lib").unwrap();
    let tempdir_path = tempdir.path();

    std::fs::create_dir(tempdir_path.join("certs")).unwrap();

    let shared = Shared {
        pueue_directory: tempdir_path.clone().to_path_buf(),
        #[cfg(not(target_os = "windows"))]
        use_unix_socket: true,
        #[cfg(not(target_os = "windows"))]
        unix_socket_path: tempdir_path.join("test.socket"),
        host: "localhost".to_string(),
        port: "51230".to_string(),
        daemon_cert: tempdir_path.join("certs").join("daemon.cert"),
        daemon_key: tempdir_path.join("certs").join("daemon.key"),
        shared_secret_path: tempdir_path.join("secret"),
    };

    let client = Client {
        read_local_logs: true,
        show_confirmation_questions: false,
        show_expanded_aliases: false,
        dark_mode: false,
        max_status_lines: Some(15),
        status_time_format: "%H:%M:%S".into(),
        status_datetime_format: "%Y-%m-%d\n%H:%M:%S".into(),
    };

    let mut groups = BTreeMap::new();
    groups.insert("default".to_string(), 1);
    groups.insert("test".to_string(), 3);

    let daemon = Daemon {
        default_parallel_tasks: 1,
        pause_group_on_failure: false,
        pause_all_on_failure: false,
        callback: None,
        callback_log_lines: 15,
        groups,
    };

    let settings = Settings {
        client,
        daemon,
        shared,
    };

    settings
        .save(&Some(tempdir_path.join("pueue.yml")))
        .context("Couldn't write pueue config to temporary directory")?;

    Ok((settings, tempdir))
}
