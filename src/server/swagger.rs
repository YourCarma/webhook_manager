use crate::errors::Successful;
use crate::server::error::ErrorMessageResponse;
use crate::server::router::api::v1::storage::{
    __path_add_response_data, __path_create_task, __path_delete_task, __path_get_task,
    __path_get_tasks, __path_update_progress, __path_websocket_handler,
};
use crate::server::router::api::v2::storage::__path_create_task_v2;
use crate::server::router::models::{
    ProgressUpdate, ResponseDataUpdate, TaskCreation, TaskCreationV2, TaskID, TaskListQuery,
};
use crate::storage::models::{Task, TaskProgress, TaskStatus};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title="Webhook Manager Service",
        version="1.0.1",
        description = "HTTP API и WebSocket API для хранения состояния длительных пользовательских задач в NOSql хранилище."
    ),
    tags(
        (
            name = "V1",
            description = "`CRUD` задач. Ключ задачи имеет формат `{user_id}:{service}:{task_id}`.",
        ),
        (
            name = "V2",
            description = "V2 `CRUD` задач. Ключ задачи имеет формат `{user_id}:{service}:{task_id}`.",
        ),
    ),

    components(
        schemas(
            Successful,
            ErrorMessageResponse,
            ProgressUpdate,
            ResponseDataUpdate,
            TaskCreation,
            TaskCreationV2,
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
       create_task_v2,
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
