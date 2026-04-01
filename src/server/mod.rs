pub mod config;
pub mod error;
pub mod router;

use std::sync::Arc;

pub mod swagger;
use axum::response::Html;
use axum::routing::{any, get, patch, post};
use axum::{Json, Router};
use axum_prometheus::PrometheusMetricLayer;
use swagger::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::storage::TaskStorage;

pub struct AppState<R>
where
    R: TaskStorage,
{
    storage: Arc<R>,
}

impl<R> AppState<R>
where
    R: TaskStorage,
{
    pub fn new(storage: Arc<R>) -> Self {
        AppState { storage }
    }
}

pub fn init_server<R>(app: AppState<R>) -> Router
where
    R: TaskStorage + Send + Sync + 'static,
{
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app_arc = Arc::new(app);
    Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(Html("<a href=\"/docs\">ДОКУМЕНТАЦИЯ</h1>")))
        .route(
            "/api/v1/storage/task",
            post(router::storage::create_task)
                .get(router::storage::get_task)
                .delete(router::storage::delete_task),
        )
        .route("/api/v1/storage/tasks", get(router::storage::get_tasks))
        .route(
            "/api/v1/storage/update_progress",
            patch(router::storage::update_progress),
        )
        .route(
            "/api/v1/storage/update_response_data",
            patch(router::storage::add_response_data),
        )
        .route(
            "/api/v1/storage/ws",
            any(router::storage::websocket_handler),
        )
        .route("/health", get(Json("OK")))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer)
        .with_state(app_arc)
}
