use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{cors, trace};

use webhook_manager::ServiceConnect;
use webhook_manager::config::ServiceConfig;
use webhook_manager::logger;
use webhook_manager::server::AppState;
use webhook_manager::storage::redis::RedisStorage;

#[tokio::main(worker_threads = 8)]
async fn main() -> anyhow::Result<()> {
    let config = ServiceConfig::new()?;
    logger::init_logger(config.logger())?;

    let storage = Arc::new(RedisStorage::connect(config.storage().redis()).await?);
    let server_app = AppState::new(storage);

    let cors_layer = cors::CorsLayer::permissive();
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let app = webhook_manager::server::init_server(server_app)
        .layer(trace_layer)
        .layer(cors_layer)
        .layer(OtelAxumLayer::default());

    let server_config = config.server();
    tracing::info!(
        address = format!("http://{}", server_config.address()),
        "Running server on"
    );
    let listener = TcpListener::bind(server_config.address()).await?;

    if let Err(err) = axum::serve(listener, app).await {
        tracing::error!(err=?err, "failed to stop http server");
    };

    Ok(())
}
