pub mod config;
pub mod error;

use getset::CopyGetters;
use redis::{AsyncCommands, AsyncIter, Client, RedisError, RedisResult, ToRedisArgs};
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
        let address = format!("redis://{}:{}", config.host().as_str(), config.port());
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
    async fn create_task(&self, key: &str, value: Task) -> SubmitResult {
        let expired_secs = self.options.expired();
        let cxt = self.client.write().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let result: RedisResult<()> = conn.set_ex(key, value, expired_secs).await;
        if let Err(err) = result {
            tracing::warn!(err=?err, "failed to get redis service connection");
            return Err(StorageError::KeyNotFound(err.to_string()));
        }

        Ok(())
    }

    async fn update_progress(&self, key: &str, data: TaskProgress) -> SubmitResult {
        let mut task_to_update = self.get_task(&key).await?;
        task_to_update.set_progress(data);
        let _ = self.set_value(&key, task_to_update).await?;
        Ok(())
    }

    async fn add_response_data(&self, key: &str, data: String) -> SubmitResult {
        let mut task_to_update = self.get_task(&key).await?;
        task_to_update.set_response_data(data);
        let _ = self.set_value(&key, task_to_update).await?;

        Ok(())
    }

    async fn get_task(&self, key: &str) -> StorageResult<Task> {
        let cxt = self.client.read().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        match conn.get(key).await {
            Ok(task) => Ok(task),
            Err(err) => {
                tracing::warn!(key=key, err=?err, "failed to get task from redis");
                Err(StorageError::from(err))
            }
        }
    }

    async fn get_tasks(&self, pattern: &str) -> StorageResult<Vec<FormattedTask>> {
        let client_keys = self.scan_values(pattern).await?;
        let mut tasks = Vec::new();
        for key in client_keys.iter() {
            let value: Task = self.get_task(key).await?;
            let ttl = self.get_ttl(key).await?;
            let result = FormattedTask {
                task: value,
                expire: ttl,
            };
            tasks.push(result);
        }
        Ok(tasks)
    }
}

impl RedisStorage {
    async fn set_value<T>(&self, key: &str, value: T) -> SubmitResult
    where
        T: ToRedisArgs + Send + Sync,
    {
        let cxt = self.client.write().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let result: RedisResult<()> = conn.set(&key, value).await;
        if let Err(err) = result {
            tracing::error!(err=?err, "Failed to set value: {key}");
            return Err(StorageError::KeyNotFound(err.to_string()));
        }
        Ok(())
    }

    async fn scan_values(&self, pattern: &str) -> StorageResult<Vec<String>> {
        let cxt = self.client.read().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let mut matched_keys: AsyncIter<String> = conn.scan_match(pattern).await?;
        let mut keys: Vec<String> = Vec::new();
        while let Some(element) = matched_keys.next_item().await {
            keys.push(element)
        }
        Ok(keys)
    }

    async fn get_ttl(&self, key: &str) -> StorageResult<u64> {
        let cxt = self.client.read().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let ttl: u64 = conn.expire_time(key).await?;
        Ok(ttl)
    }
}

#[cfg(test)]
mod test_redis {
    use super::*;

    #[test]
    fn test_add() {
        // let input_1 = 2;
        // let input_2 = 8;
        // let result = add(input_1, input_2);
        // assert_eq!(result, 10, "The addition result is incorrect.");
    }
}
