use color_eyre::eyre::Result;
use tokio::net::TcpStream;
use tracing::{Level, Span};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_tracing() -> Result<()> {
    let fmt_layer = fmt::layer().compact();

    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

pub fn make_span_with_request_id(_request: TcpStream) -> Span {
    tracing::span!(
        Level::INFO,
        "[REQUEST]",
    )
}

pub fn on_request(_request: TcpStream, _span: &Span) {
    tracing::event!(Level::INFO, "[REQUEST START]");
}

