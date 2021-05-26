use std::fs::File;
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, io::Read};

use anyhow::{anyhow, bail, Context, Result};
use tempdir::TempDir;
use tokio::io::{self, AsyncWriteExt};

use pueue_daemon_lib::run;
use pueue_lib::network::message::*;
use pueue_lib::settings::*;
use pueue_lib::state::State;

use super::network::*;
use super::sleep_ms;

async fn run_and_handle_error(pueue_dir: PathBuf) -> Result<()> {
    if let Err(err) = run(Some(pueue_dir.join("pueue.yml"))).await {
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

pub fn start_daemon(pueue_dir: &Path) -> Result<i32> {
    let path = pueue_dir.clone().to_path_buf();
    // Start/spin off the daemon and get its PID
    tokio::spawn(run_and_handle_error(path));
    let pid = get_pid(pueue_dir)?;

    // Wait a little longer for the daemon to properly start
    sleep_ms(500);

    return Ok(pid);
}

/// A simple helper, which sends a a shutdown message and waits a little.
/// After receiving the shutdown message, the daemon will exit with status 0 themselves.
pub async fn shutdown(shared: &Shared) -> Result<()> {
    send_message(shared, Message::DaemonShutdown).await?;

    sleep_ms(500);
    Ok(())
}

pub async fn get_state(shared: &Shared) -> Result<Box<State>> {
    let response = send_message(shared, Message::Status).await?;
    match response {
        Message::StatusResponse(state) => Ok(state),
        _ => bail!("Didn't get status response in get_state"),
    }
}

/// Get a daemon pid from a specific pueue directory.
/// This function gives the daemon a little time to boot up, but ultimately crashes if it takes too
/// long.
pub fn get_pid(pueue_dir: &Path) -> Result<i32> {
    let pid_file = pueue_dir.join("pueue.pid");

    // Give the daemon about 1 sec to boot and create the pid file.
    let tries = 10;
    let mut current_try = 0;

    while current_try < tries {
        // The daemon didn't create the pid file yet. Wait for 100ms and try again.
        if !pid_file.exists() {
            sleep_ms(50);
            current_try += 1;
            continue;
        }

        let mut file = File::open(&pid_file).context("Couldn't open pid file")?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .context("Couldn't write to file")?;

        // The file has been created but not yet been written to.
        if content.is_empty() {
            sleep_ms(50);
            current_try += 1;
            continue;
        }

        let pid = content
            .parse::<i32>()
            .map_err(|_| anyhow!("Couldn't parse value: {}", content))?;
        return Ok(pid);
    }

    panic!("Couldn't find pid file after about 1 sec.");
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
