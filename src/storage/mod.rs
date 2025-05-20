pub mod redis;

mod models;
mod error;

use uuid::Uuid;

use crate::storage::models::Task;
use crate::storage::error::StorageResult;


#[async_trait::async_trait]
pub trait TaskStorage {
    async fn create_task(task: Task) -> StorageResult<()>;

    async fn update_task(task_id: Uuid) ->  StorageResult<()>;

    async fn get_task(task_id: Uuid) -> Task;

    async fn get_client_tasks(client_id: Uuid) -> Vec<Task>;

}