use std::sync::Arc;

use axum::extract::{Json, State};
use axum::response::IntoResponse;
use regex::Regex;

use crate::errors::Successful;
use crate::server::AppState;
use crate::server::error::{ErrorMessageResponse, ServerError, ServerResult};
use crate::server::router::models::TaskCreationV2;
use crate::storage::TaskStorage;
use crate::storage::models::Task;

fn build_task_key_from_task(task: &Task) -> String {
    format!("{}:{}:{}", task.user_id(), task.service(), task.task_id())
}

#[inline]
fn check_key_pattern(key: &str) -> bool {
    let re = Regex::new(r"^[^:]+:[^:]+:[^:]+$").unwrap();
    re.is_match(key)
}

#[utoipa::path(
    post,
    path = "/api/v2/storage/task",
    request_body = TaskCreationV2,
    tags=["V2"],
    summary = "Создать задачу без ключа в теле запроса",
    description=r#"
## Создание новой задачи v2

Создаёт задачу в Redis. В теле запроса передаётся только `task`, а ключ хранилища
формируется сервером из полей задачи в формате `user_id:service:task_id`.

### Параметры запроса
- **task**: словарь информации по задаче
    - **task_id** (UUID4): ID задачи
    - **user_id**: ID пользователя
    - **service**: имя сервиса
    - **created_at** (timestamp, optional): Время создания задачи. Если поле не передано, сервер задаёт текущие дату и время.
    - **progress**:
        - **progress** (float)
        - **status** (string): Системный статус задачи. Один из `[ PENDING, AWAITING, PROCESSING, READY, ERROR, CANCELLED ]`
    - **updated_at** (timestamp, optional): Время обновления задачи. Если поле не передано, сервер задаёт текущие дату и время.
    - **response_data** (JSON-string): Пользовательская информация по сервису.
"#,
    responses(
        (status = 200, description = "Задача создана", body = Successful),
        (status = 400, description = "Некорректный JSON", body = ErrorMessageResponse),
        (status = 500, description = "Внутренняя ошибка хранилища", body = ErrorMessageResponse),
        (status = 503, description = "Redis или зависимый сервис недоступен", body = ErrorMessageResponse)
    ))]
pub async fn create_task_v2<R>(
    State(state): State<Arc<AppState<R>>>,
    Json(task): Json<TaskCreationV2>,
) -> ServerResult<impl IntoResponse>
where
    R: TaskStorage + Send + Sync,
{
    let key = build_task_key_from_task(task.task());
    if !check_key_pattern(&key) {
        return Err(ServerError::IvalidKeyFormat("Key format error".to_owned()));
    }
    state.storage.create_task(&key, task.task()).await?;
    Ok(Json(Successful::default()))
}

#[cfg(test)]
mod tests {
    use super::{build_task_key_from_task, check_key_pattern};

    use crate::storage::models::Task;

    #[test]
    fn builds_task_key_from_task_identity_fields() {
        let task = Task::default();

        let key = build_task_key_from_task(&task);

        assert_eq!(key, "guest:general:96366fb0-0c0f-4671-8f3f-8a98641d11ae");
    }

    #[test]
    fn accepts_generated_task_key_pattern() {
        let task = Task::default();
        let key = build_task_key_from_task(&task);

        assert!(check_key_pattern(&key));
    }
}
