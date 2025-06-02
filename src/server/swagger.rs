use crate::errors::*;
use crate::server::router::models::{ProgressUpdate, ResponseDataUpdate, TaskCreation, TaskID};
use crate::server::router::storage::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title="Webhook Manager Service",
        version="0.5.0",
        description = "Webhook Manager для управления задачами клиентов"
    ),
    tags(
        (
            name = "Задачи",
            description = "### Модуль управления задачами",
        ),
    ),

    components(
        schemas(
            Successful,
            ErrorResponse,
            ProgressUpdate,
            ResponseDataUpdate,
            TaskCreation,
            TaskID,
        ),
    ),
    paths(
       create_task,
       add_response_data,
       get_task,
       get_tasks,
       update_progress,
       delete_task,
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

impl SwaggerExample for ErrorResponse {
    type Example = Self;

    fn example(value: Option<&str>) -> Self::Example {
        let msg = value.unwrap_or("bad client request");
        ErrorResponse::new(400, "Bad request", msg)
    }
}
