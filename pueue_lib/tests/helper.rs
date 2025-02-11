use portpicker::pick_unused_port;
use pueue_lib::settings::*;
use tempfile::{Builder, TempDir};

pub fn get_shared_settings(
    #[cfg_attr(target_os = "windows", allow(unused_variables))] use_unix_socket: bool,
) -> (Shared, TempDir) {
    // Create a temporary directory used for testing.
    let tempdir = Builder::new().prefix("pueue_lib-").tempdir().unwrap();
    let tempdir_path = tempdir.path();

    std::fs::create_dir(tempdir_path.join("certs")).unwrap();

    let shared_settings = Shared {
        pueue_directory: Some(tempdir_path.to_path_buf()),
        runtime_directory: Some(tempdir_path.to_path_buf()),
        alias_file: None,
        #[cfg(not(target_os = "windows"))]
        use_unix_socket,
        #[cfg(not(target_os = "windows"))]
        unix_socket_path: None,
        #[cfg(not(target_os = "windows"))]
        unix_socket_permissions: Some(0o700),
        pid_path: None,
        host: "localhost".to_string(),
        port: pick_unused_port()
            .expect("There should be a free port")
            .to_string(),
        daemon_cert: Some(tempdir_path.join("certs").join("daemon.cert")),
        daemon_key: Some(tempdir_path.join("certs").join("daemon.key")),
        shared_secret_path: Some(tempdir_path.join("secret")),
    };

    (shared_settings, tempdir)
}
