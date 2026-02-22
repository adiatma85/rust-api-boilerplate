use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

pub fn init_logger() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();
}
