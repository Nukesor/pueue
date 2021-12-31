use tempdir::TempDir;

use portpicker::pick_unused_port;
use pueue_lib::settings::*;

pub fn get_shared_settings() -> (Shared, TempDir) {
    // Create a temporary directory used for testing.
    let tempdir = TempDir::new("pueue_lib").unwrap();
    let tempdir_path = tempdir.path();

    std::fs::create_dir(tempdir_path.join("certs")).unwrap();

    let shared_settings = Shared {
        pueue_directory: tempdir_path.clone().to_path_buf(),
        #[cfg(not(target_os = "windows"))]
        use_unix_socket: true,
        #[cfg(not(target_os = "windows"))]
        unix_socket_path: tempdir_path.join("test.socket"),
        host: "localhost".to_string(),
        port: pick_unused_port()
            .expect("There should be a free port")
            .to_string(),
        daemon_cert: tempdir_path.join("certs").join("daemon.cert"),
        daemon_key: tempdir_path.join("certs").join("daemon.key"),
        shared_secret_path: tempdir_path.join("secret"),
    };

    (shared_settings, tempdir)
}
