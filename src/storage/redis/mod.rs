pub mod config;
pub mod error;

use getset::CopyGetters;
use redis::{AsyncCommands, AsyncIter, Client, RedisError, RedisResult, ScanOptions, ToRedisArgs};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ServiceConnect;
use crate::storage::TaskStorage;
use crate::storage::error::{StorageError, StorageResult, SubmitResult};
use crate::storage::models::{FormattedTask, Task, TaskProgress};
use crate::storage::redis::config::RedisConfig;

#[derive(Clone, CopyGetters)]
pub struct RedisStorage {
    options: Arc<RedisConfig>,
    client: Arc<RwLock<Client>>,
}

#[async_trait::async_trait]
impl ServiceConnect for RedisStorage {
    type Config = RedisConfig;
    type Error = RedisError;
    type Client = RedisStorage;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        let address = config.address();
        let client = Client::open(address.clone())?;
        // TODO: Need to add log message of successful connection
        tracing::info!(address = address);
        Ok(RedisStorage {
            options: Arc::new(config.to_owned()),
            client: Arc::new(RwLock::new(client)),
        })
    }
}

#[async_trait::async_trait]
impl TaskStorage for RedisStorage {
    async fn create_task(&self, key: &str, value: &Task) -> SubmitResult {
        let expired_secs = self.options.expired();
        let cxt = self.client.write().await;
        tracing::info!(task=?value, "Creating task: {key}");
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let result: RedisResult<()> = conn.set_ex(key, value, expired_secs).await;
        if let Err(err) = result {
            tracing::warn!(err=?err, "failed to get redis service connection");
            return Err(StorageError::KeyNotFound(err.to_string()));
        }
        tracing::info!("Task created!");
        Ok(())
    }

    async fn update_progress(&self, key: &str, data: &TaskProgress) -> SubmitResult {
        tracing::info!(task=?data, "Updating progress: {key}");
        let mut task_to_update = self.get_task(&key).await?;
        task_to_update.set_progress(data.clone());
        let _ = self.set_value(&key, task_to_update).await?;
        tracing::info!("Progress updated!");
        Ok(())
    }

    async fn add_response_data(&self, key: &str, data: &String) -> SubmitResult {
        tracing::info!(task=?data, "Updating response data: {key}");
        let mut task_to_update = self.get_task(&key).await?;
        task_to_update.set_response_data(data.to_owned());
        let _ = self.set_value(&key, task_to_update).await?;
        tracing::info!("Response data updated!");
        Ok(())
    }

    async fn get_task(&self, key: &str) -> StorageResult<Task> {
        tracing::info!("Getting task: {key}");
        let cxt = self.client.read().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        match conn.get(key).await {
            Ok(task) => {
                tracing::debug!(task=?task, "Found task");
                Ok(task)
            }
            Err(err) => {
                tracing::warn!(key=key, err=?err, "failed to get task from redis");
                Err(StorageError::from(err))
            }
        }
    }

    async fn get_tasks(&self, pattern: &str) -> StorageResult<Vec<FormattedTask>> {
        tracing::info!("Getting tasks: {pattern}");
        let client_keys = self.scan_values(pattern).await?;
        let mut tasks = Vec::new();
        for key in client_keys.iter() {
            let value: Task = match self.get_task(key).await {
                Ok(task) => task,
                Err(err) => {
                    tracing::error!(key=key, err=?err, "failed to get task from redis");
                    continue;
                }
            };
            let ttl = self.get_ttl(key).await?;
            let mut result = FormattedTask::default();
            result.set_task(value);
            result.set_expire(ttl);
            tasks.push(result);
        }
        tracing::debug!(tasks=?tasks, "Found client tasks: ");
        Ok(tasks)
    }

    async fn delete_task(&self, key: &str) -> SubmitResult {
        let _ = self.delete_key(key).await?;
        Ok(())
    }
}

impl RedisStorage {
    async fn set_value<T>(&self, key: &str, value: T) -> SubmitResult
    where
        T: ToRedisArgs + Send + Sync,
    {
        let cxt = self.client.write().await;
        let expired_secs = self.options.expired();
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let result: RedisResult<()> = conn.set_ex(&key, value, expired_secs).await;
        if let Err(err) = result {
            tracing::error!(err=?err, "Failed to set value: {key}");
            return Err(StorageError::KeyNotFound(err.to_string()));
        }
        Ok(())
    }

    async fn scan_values(&self, pattern: &str) -> StorageResult<Vec<String>> {
        const SCAN_COUNT: usize = 1000;
        let cxt = self.client.read().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let opts = ScanOptions::default()
            .with_pattern(pattern)
            .with_count(SCAN_COUNT);
        let mut matched_keys: AsyncIter<String> = conn.scan_options(opts).await?;
        let mut keys: Vec<String> = Vec::new();
        while let Some(element) = matched_keys.next_item().await {
            keys.push(element)
        }
        Ok(keys)
    }

    async fn get_ttl(&self, key: &str) -> StorageResult<i64> {
        let cxt = self.client.read().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let ttl: i64 = conn.ttl(key).await?;
        Ok(ttl)
    }

    async fn delete_key(&self, key: &str) -> SubmitResult {
        let cxt = self.client.write().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let key_exists = conn.exists(key).await?;
        match key_exists {
            true => {
                let result: RedisResult<()> = conn.del(key).await;
                if let Err(err) = result {
                    tracing::error!(err=?err, "Failed to delete value: {key}");
                    return Err(StorageError::AnotherError(err.to_string()));
                }
                Ok(())
            }
            false => {
                tracing::error!("Key not found: {key}");
                return Err(StorageError::KeyNotFound("Key not found".to_owned()));
            }
        }
    }
}

#[cfg(test)]
mod test_redis {

    use crate::ServiceConnect;
    use crate::config::ServiceConfig;
    use crate::storage::TaskStorage;
    use crate::storage::models::{Task, TaskProgress, TaskStatus};
    use crate::storage::redis::RedisStorage;
    use redis::AsyncCommands;

    #[tokio::test]
    async fn test_connection() -> Result<(), anyhow::Error> {
        let s_config = ServiceConfig::new()?;
        let redis_config = s_config.storage();
        let redis = RedisStorage::connect(redis_config).await?;
        let cxt = redis.client.write().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let connected: String = conn.ping().await?;
        assert_eq!(connected, "PONG");
        Ok(())
    }

    #[tokio::test]
    async fn test_create_task() -> Result<(), anyhow::Error> {
        let s_config = ServiceConfig::new()?;
        let redis_config = s_config.storage();
        let redis = RedisStorage::connect(redis_config).await?;
        let key = "test_user1:test_service1:test_task1";
        let task = Task::default();
        let created = redis.create_task(key, &task).await?;
        assert_eq!(created, ());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_task() -> Result<(), anyhow::Error> {
        let s_config = ServiceConfig::new()?;
        let redis_config = s_config.storage();
        let redis = RedisStorage::connect(redis_config).await?;
        let key = "test_user1:test_service1:test_task1";
        let task = Task::default();
        let result = redis.get_task(key).await?;
        assert_eq!(result, task);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_progress() -> Result<(), anyhow::Error> {
        let s_config = ServiceConfig::new()?;
        let redis_config = s_config.storage();
        let redis = RedisStorage::connect(redis_config).await?;
        let key = "test_user1:test_service1:test_task1";
        let mut progress = TaskProgress::default();
        progress.set_progress(100.0).set_status(TaskStatus::Ready);
        let _ = redis.update_progress(key, &progress).await?;
        let result = redis.get_task(key).await?;
        assert_eq!(progress, *result.progress());
        Ok(())
    }

    #[tokio::test]
    async fn test_add_response() -> Result<(), anyhow::Error> {
        let s_config = ServiceConfig::new()?;
        let redis_config = s_config.storage();
        let redis = RedisStorage::connect(redis_config).await?;
        let key = "test_user1:test_service1:test_task1";
        let test_response_data = "{\n    \"file_url\" : \"www.example.com\"\n}".to_string();
        let _ = redis.add_response_data(key, &test_response_data).await?;
        let result = redis.get_task(key).await?;
        assert_eq!(test_response_data, *result.response_data());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_client_tasks() -> Result<(), anyhow::Error> {
        let s_config = ServiceConfig::new()?;
        let redis_config = s_config.storage();
        let redis = RedisStorage::connect(redis_config).await?;
        let key_task = "test_user1:test_service1:test_task1";
        let key_alt_task = "test_user1:test_service1:test_task2";
        let task = Task::default();
        let ley_alt_task = Task::default();
        let _ = redis.create_task(key_task, &task).await?;
        let _ = redis.create_task(key_alt_task, &ley_alt_task).await?;
        let client_tasks_pattern = "test_user1:test_service1:*";
        let client_tasks = redis.get_tasks(client_tasks_pattern).await?;
        println!("{:?}", client_tasks);
        assert_eq!(client_tasks.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_task() -> Result<(), anyhow::Error> {
        let s_config = ServiceConfig::new()?;
        let redis_config = s_config.storage();
        let redis = RedisStorage::connect(redis_config).await?;
        let key = "test_user1:test_service1:test_task1";
        let empty_result = redis.get_task(key).await?;
        tracing::trace!(empty_result=?empty_result);
        let _ = redis.delete_key(key).await?;
        Ok(())
    }
}
