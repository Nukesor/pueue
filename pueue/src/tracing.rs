use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    EnvFilter, Layer, field::MakeExt, fmt::time::ChronoLocal, layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::internal_prelude::*;

pub fn install_tracing(verbosity: u8) -> Result<()> {
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

    type GenericLayer<S> = Box<dyn Layer<S> + Send + Sync>;
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
        .from_env()
        .wrap_err("RUST_LOG env variable is invalid")?;

    tracing_subscriber::Registry::default()
        .with(fmt_layer.with_filter(filter_layer))
        .with(tracing_error::ErrorLayer::default())
        .init();

    Ok(())
}
