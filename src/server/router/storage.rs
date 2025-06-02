use std::time::Duration;
use regex::Regex;
use tokio::time;
use std::sync::Arc;

use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::{Json, Query, State};
use axum::response::IntoResponse;

use crate::errors::{ErrorResponse, Successful};
use crate::server::AppState;
use crate::server::error::{ServerError, ServerResult};
use crate::server::router::models::{ProgressUpdate, ResponseDataUpdate, TaskCreation};
use crate::storage::TaskStorage;
use crate::storage::models::Task;

use super::models::TaskID;

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

#[utoipa::path(
    post,
    path = "/storage/task",
    request_body = TaskCreation,
    tags=["Задачи"],
    description="
    ### Создание задачи\n
    ### Входные данные:
    **key**: Ключ для хранилища формата `client_id:service_name:task_id`
    **task**: Объект задачи типа
    ",
    responses(
        (status = 201, body = Successful),
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ))]
pub async fn create_task<R>(
    State(state): State<Arc<AppState<R>>>,
    Json(task): Json<TaskCreation>,
) -> ServerResult<impl IntoResponse>
where
    R: TaskStorage + Send + Sync,
{
    let key = task.key();
    let res = match check_key_pattern(key) {
        true => {
            let task = task.task();
            let result = state.storage.create_task(key, task).await?;
            Ok(Json(Successful::default()))
        }
        false => return Err(ServerError::IvalidKeyFormat("Key format error".to_owned())),
    };

    res
}

#[utoipa::path(
    get,
    path = "/storage/task",
    tags=["Задачи"],
    params(
        (
            "key" = &str,
            Query,
            description = "",
            example = "384f4d80-4ed6-4032-8569-f02fd5e1afb9:service_name:384f4d80-4ed6-4032-2569-f02fd5e1afb9",
        ),
    ),
    responses(
        (status = 200, description = "Ok", body=Task),
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
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
    path = "/storage/update_progress",
    tags=["Задачи"],
    request_body = ProgressUpdate,
    responses(
        (status = 200, body = Successful),
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
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
            let result = state.storage.update_progress(key, updated_progress).await?;
            Ok(Json(Successful::default()))
        }
        false => Err(ServerError::IvalidKeyFormat("Key format error".to_owned())),
    }
}

#[utoipa::path(
    patch,
    path = "/storage/update_response_data",
    tags=["Задачи"],
    request_body = ResponseDataUpdate,
    responses(
        (status = 200, body = Successful),
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
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
            let result = state
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
    path = "/storage/task",
    tags=["Задачи"],
    params(
        (
            "key" = &str,
             Query,
            description = "ID of task to get",
            example = "384f4d80-4ed6-4032-8569-f02fd5e1afb9:service_name:384f4d80-4ed6-4032-2569-f02fd5e1afb9",
        ),
    ),
    responses(
        (status = 201, body = Successful),
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
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
            let result = state.storage.delete_task(key).await?;
            Ok(Json(Successful::default()))
        }
        false => Err(ServerError::IvalidKeyFormat("Key format error".to_owned())),
    }
}

#[utoipa::path(
    get,
    path = "/storage/tasks",
    tags=["Задачи"],
    params(
        (
            "key" = &str,
             Query,
            description = "ID of task to get",
            example = "384f4d80-4ed6-4032-8569-f02fd5e1afb9:*",
        ),
    ),
    responses(
        (status = 200, body = Vec<Task>),
        (status = 400, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ))]
pub async fn get_tasks<R>(
    State(state): State<Arc<AppState<R>>>,
    Query(key): Query<TaskID>,
) -> ServerResult<impl IntoResponse>
where
    R: TaskStorage + Send + Sync,
{
    let key = key.key();
    match check_client_key_pattern(key) {
        true => {
            let result = state.storage.get_tasks(key).await?;
            Ok(Json(result))
        }
        false => Err(ServerError::IvalidKeyFormat("Key format error".to_owned())),
    }
}

pub async fn websocket_handler<R>(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState<R>>>,
) -> impl IntoResponse
where
    R: TaskStorage + Send + Sync +  'static,
{
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket<R>(mut socket: WebSocket, state: Arc<AppState<R>>)
where
    R: TaskStorage + Send + Sync,
{
    if let Err(e) = socket
        .send(Message::Text("Hello from the server!".into()))
        .await
    {
        eprintln!("Error sending message: {}", e);
        return;
    }
    if let Some(Ok(msg)) = socket.recv().await{
         match msg {
            Message::Text(msg) => {
                tracing::debug!(msg=?msg,"Received message:");
                if let true = check_client_key_pattern(&msg){
                    let mut interval = time::interval(Duration::from_secs(1));
                    loop {
                        interval.tick().await;
                        let tasks = match state.storage.get_tasks(&msg.to_owned()).await{
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
                            break;
                        }
                        if let Some(Ok(Message::Close(_)))  = socket.recv().await{
                            tracing::error!("Closing WebSocket connection.");
                            break;
                        }
                        
                    }

                }
                
             }
            Message::Close(_) => {
                tracing::error!("Closing WebSocket connection.");
            }
            _ => {}
         }
    }
    // while let Some(Ok(msg)) = socket.recv().await {
    //     match msg {
    //         Message::Text(msg) => {
    //             println!("Received message: {}", msg);
    //             let tasks = match state.storage.get_tasks(&msg.to_owned()).await{
    //                 Ok(tasks) => tasks,
    //                 Err(err) => {
    //                     tracing::error!(err=?err, "Error in websocket:");
    //                     continue;
    //                 }
    //             };
    //             if let Err(e) = socket
    //                 .send(Message::Text(serde_json::to_string(&tasks).unwrap().into()))
    //                 .await
    //             {
    //                 eprintln!("Error sending message: {}", e);
    //             }
    //         }
    //         Message::Close(_) => {
    //             println!("Closing WebSocket connection.");
    //             break;
    //         }
    //         _ => {}
    //     }
    // }
}
