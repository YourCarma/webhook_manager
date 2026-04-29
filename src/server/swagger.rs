use crate::errors::Successful;
use crate::server::error::ErrorMessageResponse;
use crate::server::router::models::{
    ProgressUpdate, ResponseDataUpdate, TaskCreation, TaskID, TaskListQuery,
};
use crate::server::router::storage::*;
use crate::storage::models::{Task, TaskProgress, TaskStatus};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title="Webhook Manager Service",
        version="0.5.0",
        description = "HTTP API и WebSocket API для хранения состояния длительных пользовательских задач в NOSql хранилище."
    ),
    tags(
        (
            name = "Задачи",
            description = "`CRUD` задач. Ключ задачи имеет формат `{user_id}:{service}:{task_id}`.",
        ),
    ),

    components(
        schemas(
            Successful,
            ErrorMessageResponse,
            ProgressUpdate,
            ResponseDataUpdate,
            TaskCreation,
            TaskID,
            TaskListQuery,
            Task,
            TaskProgress,
            TaskStatus,
        ),
    ),
    paths(
       create_task,
       add_response_data,
       get_task,
       get_tasks,
       update_progress,
       delete_task,
       websocket_handler,
    )
)]
pub(super) struct ApiDoc;

pub trait SwaggerExample {
    type Example;

    fn example(value: Option<&str>) -> Self::Example;
}

impl SwaggerExample for Successful {
    type Example = Self;

    fn example(value: Option<&str>) -> Self::Example {
        let msg = value.unwrap_or("Done");
        Successful::new(200, msg)
    }
}
