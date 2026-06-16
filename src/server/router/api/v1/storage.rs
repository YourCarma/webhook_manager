use axum::http::HeaderMap;
use regex::Regex;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Json, Query, State};
use axum::response::IntoResponse;

use crate::errors::Successful;
use crate::server::AppState;
use crate::server::error::ErrorMessageResponse;
use crate::server::error::{ServerError, ServerResult};
use crate::server::router::models::{
    ProgressUpdate, ResponseDataUpdate, TaskCreation, TaskID, TaskListQuery,
};
use crate::storage::TaskStorage;
use crate::storage::models::Task;

#[inline]
fn check_key_pattern(key: &str) -> bool {
    // Checking pattern like client_id:service_name:task_id
    let re = Regex::new(r"^[^:]+:[^:]+:[^:]+$").unwrap();
    re.is_match(key)
}

fn check_client_key_pattern(key: &str) -> bool {
    // Checking pattern like client_id:service_name:task_id
    let re = Regex::new(r"^[^:]+:\*$").unwrap();
    re.is_match(key)
}

fn select_user_id(query_user_id: Option<&str>, headers: &HeaderMap) -> ServerResult<String> {
    let header_user_id = headers
        .get("X-User-ID")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let query_user_id = query_user_id
        .map(str::trim)
        .filter(|value| !value.is_empty());

    header_user_id
        .or(query_user_id)
        .map(str::to_owned)
        .ok_or_else(|| {
            ServerError::IvalidKeyFormat(
                "user_id is required in query or X-User-ID header".to_owned(),
            )
        })
}

#[utoipa::path(
    post,
    path = "/api/v1/storage/task",
    request_body = TaskCreation,
    tags=["V1"],
    summary = "Создать задачу",
    description=r#"
## Создание новой задачи

### Создание задачи для хранища Redis

### Параметры запроса
- **key** (string): Ключ для хранилища формата `user_id:service_name:task_id`
- **task**: словарь информации по задаче
    - **created_at** (timestamp, optional): Время создания задачи. Если поле не передано, сервер задаёт текущие дату и время.
    - **progress**:
        - **progress** (float)
        - **status** (string): Системный статус задачи. Один из `[ PENDING, AWAITING, PROCESSING, READY, ERROR ]`"
    - **task_id** (UUID4): ID задачи
    - **user_id** (UUID4): ID пользователя
    - **updated_at** (timestamp, optional): Время обновления задачи. Если поле не передано, сервер задаёт текущие дату и время.
    - **response data** (JSON-string): Пользовательская информация по сервису.
"#,
    responses(
        (status = 200, description = "Задача создана", body = Successful),
        (status = 400, description = "Некорректный JSON или query-параметры", body = ErrorMessageResponse),
        (status = 422, description = "Ключ не соответствует формату `{user_id}:{service}:{task_id}`", body = ErrorMessageResponse),
        (status = 500, description = "Внутренняя ошибка хранилища", body = ErrorMessageResponse),
        (status = 503, description = "Redis или зависимый сервис недоступен", body = ErrorMessageResponse)
    ))]
#[deprecated(note = "Use POST /api/v2/storage/task instead")]
pub async fn create_task<R>(
    State(state): State<Arc<AppState<R>>>,
    Json(task): Json<TaskCreation>,
) -> ServerResult<impl IntoResponse>
where
    R: TaskStorage + Send + Sync,
{
    let key = task.key();
    match check_key_pattern(key) {
        true => {
            let task = task.task();
            state.storage.create_task(key, task).await?;
            Ok(Json(Successful::default()))
        }
        false => Err(ServerError::IvalidKeyFormat("Key format error".to_owned())),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/storage/task",
    tags=["V1"],
    summary = "Получить задачу по ключу",
    description = "Возвращает одну задачу по ключу формата `{user_id}:{service}:{task_id}`.",
    params(
        (
            "key" = &str,
            Query,
            description = "Ключ задачи в формате `{user_id}:{service}:{task_id}`.",
            example = "guest:general:384f4d80-4ed6-4032-2569-f02fd5e1afb9",
        ),
    ),
    responses(
        (status = 200, description = "Задача найдена", body=Task),
        (status = 400, description = "Некорректные query-параметры", body = ErrorMessageResponse),
        (status = 404, description = "Задача с таким ключом не найдена", body = ErrorMessageResponse),
        (status = 422, description = "Ключ не соответствует формату `{user_id}:{service}:{task_id}`", body = ErrorMessageResponse),
        (status = 500, description = "Внутренняя ошибка хранилища", body = ErrorMessageResponse),
        (status = 503, description = "Redis или зависимый сервис недоступен", body = ErrorMessageResponse)
    ))]
pub async fn get_task<R>(
    State(state): State<Arc<AppState<R>>>,
    Query(key): Query<TaskID>,
) -> ServerResult<impl IntoResponse>
where
    R: TaskStorage + Send + Sync,
{
    let key = key.key();
    match check_key_pattern(key) {
        true => {
            let result = state.storage.get_task(key).await?;
            Ok(Json(result))
        }
        false => Err(ServerError::IvalidKeyFormat("Key format error".to_owned())),
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/storage/update_progress",
    tags=["V1"],
    request_body = ProgressUpdate,
    summary = "Обновить прогресс задачи",
    description=r#"
## Обновление прогресса задачи

### Обновляет (заменяет) текущий прогресс в задаче

### Параметры запроса
- **key** (string): Ключ для хранилища формата `user_id:service_name:task_id`
- **progress**:
    - **progress** (float)
    - **status** (string): Системный статус задачи. Один из `[ PENDING, AWAITING, PROCESSING, READY, ERROR]`"
    
"#,
    responses(
        (status = 200, description = "Прогресс обновлён", body = Successful),
        (status = 400, description = "Некорректный JSON", body = ErrorMessageResponse),
        (status = 404, description = "Задача с таким ключом не найдена", body = ErrorMessageResponse),
        (status = 422, description = "Ключ не соответствует формату `{user_id}:{service}:{task_id}`", body = ErrorMessageResponse),
        (status = 500, description = "Внутренняя ошибка хранилища", body = ErrorMessageResponse),
        (status = 503, description = "Redis или зависимый сервис недоступен", body = ErrorMessageResponse)
    ))]
pub async fn update_progress<R>(
    State(state): State<Arc<AppState<R>>>,
    Json(progress): Json<ProgressUpdate>,
) -> ServerResult<impl IntoResponse>
where
    R: TaskStorage + Send + Sync,
{
    let key = progress.key();
    match check_key_pattern(key) {
        true => {
            let updated_progress = progress.progress();
            state.storage.update_progress(key, updated_progress).await?;
            Ok(Json(Successful::default()))
        }
        false => Err(ServerError::IvalidKeyFormat("Key format error".to_owned())),
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/storage/update_response_data",
    tags=["V1"],
    request_body = ResponseDataUpdate,
    summary = "Обновить response_data задачи",
    description=r#"
## Обновление пользовательской информации сервиса

### Обновляет (заменяет) служебную инфомормацию сервиса

### Параметры запроса
- **key** (string): Ключ для хранилища формата `user_id:service_name:task_id`
- **response data** (JSON-string): Пользовательская информация по сервису.
    
"#,
    responses(
        (status = 200, description = "response_data обновлено", body = Successful),
        (status = 400, description = "Некорректный JSON", body = ErrorMessageResponse),
        (status = 404, description = "Задача с таким ключом не найдена", body = ErrorMessageResponse),
        (status = 422, description = "Ключ не соответствует формату `{user_id}:{service}:{task_id}`", body = ErrorMessageResponse),
        (status = 500, description = "Внутренняя ошибка хранилища", body = ErrorMessageResponse),
        (status = 503, description = "Redis или зависимый сервис недоступен", body = ErrorMessageResponse)
    ))]
pub async fn add_response_data<R>(
    State(state): State<Arc<AppState<R>>>,
    Json(response_data): Json<ResponseDataUpdate>,
) -> ServerResult<impl IntoResponse>
where
    R: TaskStorage + Send + Sync,
{
    let key = response_data.key();
    match check_key_pattern(key) {
        true => {
            let updated_response_data = response_data.response_data();
            state
                .storage
                .add_response_data(key, updated_response_data)
                .await?;
            Ok(Json(Successful::default()))
        }
        false => Err(ServerError::IvalidKeyFormat("Key format error".to_owned())),
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/storage/task",
    tags=["V1"],
    summary = "Удалить задачу",
    description = "Удаляет задачу по ключу формата `{user_id}:{service}:{task_id}`.",
    params(
        (
            "key" = &str,
             Query,
            description = "Ключ задачи в формате `{user_id}:{service}:{task_id}`.",
            example = "guest:general:384f4d80-4ed6-4032-2569-f02fd5e1afb9",
        ),
    ),
    responses(
        (status = 200, description = "Задача удалена", body = Successful),
        (status = 400, description = "Некорректные query-параметры", body = ErrorMessageResponse),
        (status = 404, description = "Задача с таким ключом не найдена", body = ErrorMessageResponse),
        (status = 422, description = "Ключ не соответствует формату `{user_id}:{service}:{task_id}`", body = ErrorMessageResponse),
        (status = 500, description = "Внутренняя ошибка хранилища", body = ErrorMessageResponse),
        (status = 503, description = "Redis или зависимый сервис недоступен", body = ErrorMessageResponse)
    ))]
pub async fn delete_task<R>(
    State(state): State<Arc<AppState<R>>>,
    Query(key): Query<TaskID>,
) -> ServerResult<impl IntoResponse>
where
    R: TaskStorage + Send + Sync,
{
    let key = key.key();
    match check_key_pattern(key) {
        true => {
            state.storage.delete_task(key).await?;
            Ok(Json(Successful::default()))
        }
        false => Err(ServerError::IvalidKeyFormat("Key format error".to_owned())),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/storage/tasks",
    tags=["V1"],
    summary = "Получить задачи пользователя",
    description = "Возвращает список задач пользователя. `user_id` можно передать в query или в заголовке `X-User-ID`; если указаны оба значения, используется заголовок.",
    params(
        (
            "user_id" = Option<String>,
             Query,
            description = "ID пользователя. Используется, если заголовок `X-User-ID` отсутствует или пустой.",
            example = "guest",
        ),
        (
            "X-User-ID" = Option<String>,
             Header,
            description = "ID пользователя. Имеет приоритет над query-параметром `user_id`.",
            example = "guest",
        ),
    ),
    responses(
        (status = 200, description = "Список задач пользователя", body = Vec<Task>),
        (status = 400, description = "Некорректные query-параметры", body = ErrorMessageResponse),
        (status = 404, description = "Задачи пользователя не найдены", body = ErrorMessageResponse),
        (status = 422, description = "`user_id` не передан ни в query, ни в `X-User-ID`", body = ErrorMessageResponse),
        (status = 500, description = "Внутренняя ошибка хранилища", body = ErrorMessageResponse),
        (status = 503, description = "Redis или зависимый сервис недоступен", body = ErrorMessageResponse)
    ))]
pub async fn get_tasks<R>(
    State(state): State<Arc<AppState<R>>>,
    headers: HeaderMap,
    Query(query): Query<TaskListQuery>,
) -> ServerResult<impl IntoResponse>
where
    R: TaskStorage + Send + Sync,
{
    let user_id = select_user_id(query.user_id().as_deref(), &headers)?;
    let user_tasks = format!("{}:*", user_id);
    let result = state.storage.get_tasks(&user_tasks).await?;
    Ok(Json(result))
}

async fn process_message<R>(mut socket: WebSocket, msg: &str, state: Arc<AppState<R>>)
where
    R: TaskStorage + Send + Sync,
{
    let mut interval = time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let tasks = match state.storage.get_tasks(msg).await{
                    Ok(tasks) => tasks,
                    Err(err) => {
                        tracing::error!(err=?err, "Error in websocket:");
                        continue;
                    }
                };
                if let Err(error) = socket
                    .send(Message::Text(serde_json::to_string(&tasks).unwrap().into()))
                    .await
                    {
                        tracing::error!(error=?error, "Error sending message");
                        return ;
                    }
            }

            Some(Ok(result)) = socket.recv() => {
                if let Message::Close(_) = result {
                    break
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::select_user_id;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn uses_query_when_header_is_missing() {
        let headers = HeaderMap::new();

        let user_id = select_user_id(Some("query-user"), &headers).unwrap();

        assert_eq!(user_id, "query-user");
    }

    #[test]
    fn uses_header_when_query_is_missing() {
        let mut headers = HeaderMap::new();
        headers.insert("X-User-ID", HeaderValue::from_static("header-user"));

        let user_id = select_user_id(None, &headers).unwrap();

        assert_eq!(user_id, "header-user");
    }

    #[test]
    fn prefers_header_over_query() {
        let mut headers = HeaderMap::new();
        headers.insert("X-User-ID", HeaderValue::from_static("header-user"));

        let user_id = select_user_id(Some("query-user"), &headers).unwrap();

        assert_eq!(user_id, "header-user");
    }

    #[test]
    fn falls_back_to_query_when_header_is_blank() {
        let mut headers = HeaderMap::new();
        headers.insert("X-User-ID", HeaderValue::from_static("   "));

        let user_id = select_user_id(Some("query-user"), &headers).unwrap();

        assert_eq!(user_id, "query-user");
    }

    #[test]
    fn returns_error_when_both_sources_are_missing() {
        let headers = HeaderMap::new();

        let error = select_user_id(None, &headers).unwrap_err();

        assert!(matches!(
            error,
            crate::server::error::ServerError::IvalidKeyFormat(_)
        ));
    }

    #[test]
    fn returns_error_when_both_sources_are_blank() {
        let mut headers = HeaderMap::new();
        headers.insert("X-User-ID", HeaderValue::from_static(" "));

        let error = select_user_id(Some("   "), &headers).unwrap_err();

        assert!(matches!(
            error,
            crate::server::error::ServerError::IvalidKeyFormat(_)
        ));
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/storage/ws",
    tags=["V1"],
    summary = "Подписаться на задачи пользователя через WebSocket",
    description = r#"
Открывает WebSocket-соединение для потокового получения задач пользователя.

После подключения сервер отправляет `{"status":"PING"}`. Клиент должен отправить любое текстовое
сообщение, после чего сервер раз в секунду отправляет JSON-массив задач пользователя.

Пользователь определяется по заголовку `X-User-ID`. Если заголовок не передан, используется `guest`.
"#,
    params(
        (
            "X-User-ID" = Option<String>,
            Header,
            description = "ID пользователя для подписки. Если заголовок отсутствует, используется `guest`.",
            example = "guest",
        ),
    ),
    responses(
        (status = 101, description = "WebSocket-соединение установлено"),
        (status = 400, description = "Некорректный WebSocket upgrade-запрос", body = ErrorMessageResponse)
    )
)]
pub async fn websocket_handler<R>(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState<R>>>,
) -> impl IntoResponse
where
    R: TaskStorage + Send + Sync + 'static,
{
    ws.on_upgrade(|socket| handle_socket(socket, headers, state))
}

async fn handle_socket<R>(mut socket: WebSocket, headers: HeaderMap, state: Arc<AppState<R>>)
where
    R: TaskStorage + Send + Sync + 'static,
{
    if let Err(e) = socket
        .send(Message::Text("{\"status\": \"PING\"}".into()))
        .await
    {
        eprintln!("Error sending message: {}", e);
        return;
    }
    tracing::error!(headers=?headers, "Headers");
    let user_id = headers
        .get("X-User-ID")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| "guest".to_owned());
    tracing::debug!(user_id=?user_id, "UserID: ");
    let pattern = format!("{user_id}:*");
    if let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(msg) => {
                tracing::debug!(msg=?msg,"Received message:");
                if check_client_key_pattern(&pattern) {
                    process_message(socket, &pattern, state).await;
                }
            }
            Message::Close(_) => {
                tracing::error!("Closing WebSocket connection.");
            }
            _ => {}
        }
    }
}
