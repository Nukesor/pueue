use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    EnvFilter, Layer, Registry, field::MakeExt, filter::FromEnvError, fmt::time::ChronoLocal,
    layer::SubscriberExt, util::SubscriberInitExt,
};

#[cfg(feature = "settings")]
pub mod settings {

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
}

#[allow(dead_code)]
pub fn install_tracing(verbosity: u8) -> Result<(), FromEnvError> {
    let mut pretty = false;
    let level = match verbosity {
        0 => LevelFilter::WARN,
        1 => LevelFilter::INFO,
        2 => LevelFilter::DEBUG,
        3 => LevelFilter::TRACE,
        _ => {
            pretty = true;
            LevelFilter::TRACE
        }
    };

    // tries to find local offset internally
    let timer = ChronoLocal::new("%H:%M:%S".into());

    type GenericLayer<S> = Box<dyn tracing_subscriber::Layer<S> + Send + Sync>;
    let fmt_layer: GenericLayer<_> = match pretty {
        false => Box::new(
            tracing_subscriber::fmt::layer()
                .map_fmt_fields(|f| f.debug_alt())
                .with_timer(timer)
                .with_writer(std::io::stderr),
        ),
        true => Box::new(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_timer(timer)
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(true)
                .with_level(true)
                .with_ansi(true)
                .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
                .with_writer(std::io::stderr),
        ),
    };
    let filter_layer = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env()?;

    Registry::default()
        .with(fmt_layer.with_filter(filter_layer))
        .with(tracing_error::ErrorLayer::default())
        .init();

    Ok(())
}
