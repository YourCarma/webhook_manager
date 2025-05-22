pub mod error;
pub mod models;
pub mod redis;

use crate::storage::error::{StorageResult, SubmitResult};
use crate::storage::models::{Task, TaskProgress};

#[async_trait::async_trait]
pub trait TaskStorage {
    async fn create_task(&self, key: &str, task: Task) -> SubmitResult;
    async fn update_progress(&self, key: &str, task: TaskProgress) -> SubmitResult;
    async fn add_response_data(&self, key: &str, data: String) -> SubmitResult;
    async fn get_task(&self, key: &str) -> StorageResult<Task>;
    async fn get_tasks(&self, key: &str) -> StorageResult<Vec<Task>>;
}
