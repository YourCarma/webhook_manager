pub mod config;
pub mod error;
pub mod router;

use std::sync::Arc;

pub mod swagger;
use axum::Router;
use axum::routing::{any, get, patch, post};
use axum_prometheus::PrometheusMetricLayer;
use swagger::ApiDoc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use axum::response::Html;  

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
        .route(
            "/",
            get(Html("<a href=\"/docs\">ДОКУМЕНТАЦИЯ</h1>"))
        )
        .route(
            "/storage/task",
            post(router::storage::create_task)
                .get(router::storage::get_task)
                .delete(router::storage::delete_task),
        )
        .route("/storage/tasks", get(router::storage::get_tasks))
        .route(
            "/storage/update_progress",
            patch(router::storage::update_progress),
        )
        .route(
            "/storage/update_response_data",
            patch(router::storage::add_response_data),
        )
        .route("/ws", any(router::storage::websocket_handler))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer)
        .with_state(app_arc)
}
