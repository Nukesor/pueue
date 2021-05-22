use std::fs::File;
use std::path::Path;
use std::{collections::BTreeMap, io::Read};

use tempdir::TempDir;

use pueue_lib::settings::*;

/// Get a daemon pid from a specific pueue directory.
/// This function gives the daemon a little time to boot up, but ultimately crashes if it takes too
/// long.
pub fn get_pid(pueue_dir: &Path) -> i32 {
    let pid_file = pueue_dir.join("pueue.pid");

    // Give the daemon about 1 sec to boot and create the pid file.
    let tries = 10;
    let mut current_try = 0;

    while current_try < tries {
        // The daemon didn't create the pid file yet. Wait for 100ms and try again.
        if !pid_file.exists() {
            std::thread::sleep(std::time::Duration::from_millis(100));
            current_try += 1;
            continue;
        }

        let mut file = File::open(&pid_file).expect("Couldn't open pid file");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Couldn't write to file");

        // The file has been created but not yet been written to.
        if content.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(100));
            current_try += 1;
            continue;
        }

        return content
            .parse::<i32>()
            .expect(&format!("Couldn't parse value: {}", content));
    }

    panic!("Couldn't find pid file after about 1 sec.");
}

pub fn get_settings() -> (Settings, TempDir) {
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
        .expect("Couldn't write pueue config to temporary directory");

    (settings, tempdir)
}
