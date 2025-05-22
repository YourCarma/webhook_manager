pub mod redis;

mod models;
mod error;

use uuid::Uuid;

use crate::storage::models::{Task, TaskProgress};
use crate::storage::error::StorageResult;


#[async_trait::async_trait]
pub trait TaskStorage {
    async fn create_task(&self, key: &str, task: Task) -> StorageResult<()>;

    async fn update_progress(&self, key: &str, task: TaskProgress) -> StorageResult<()>;

    async fn add_response_data(&self, key: &str, data: String) -> StorageResult<()>;

    async fn get_task(&self, key: &str) -> Option<Task>;
    
    async fn get_client_tasks(&self, key: &str) -> StorageResult<Vec<Task>>;
}