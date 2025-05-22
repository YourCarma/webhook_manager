pub mod config;
pub mod error;

use getset::{CopyGetters, Getters, Setters};
use redis::{AsyncCommands, Client, FromRedisValue, RedisError, RedisResult, ToRedisArgs};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ServiceConnect;
use crate::storage::TaskStorage;
use crate::storage::error::{StorageError, StorageResult, SubmitResult};
use crate::storage::models::{Task, TaskProgress};
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
        let client = Client::open(address)?;
        // TODO: Need to add log message of successful connection
        // tracing::info!(address=address)
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

    async fn update_progress(&self, key: &str, value: TaskProgress) -> SubmitResult {
        let cxt = self.client.write().await;
        let mut conn = cxt.get_multiplexed_tokio_connection().await?;
        let mut task: Task = match conn.get(&key).await {
            Ok(task) => task,
            Err(err) => {
                tracing::error!(err=?err, "trying to get existing task");
                return Err(StorageError::TaskNotFound(err.to_string()));
            }
        };

        task.set_progress(value);
        let result: RedisResult<()> = conn.set(&key, task).await;
        if let Err(err) = result {
            tracing::error!(err=?err, "Failed to update an progress");
            return Err(StorageError::KeyNotFound(err.to_string()));
        }

        Ok(())
    }

    async fn add_response_data(&self, key: &str, data: String) -> SubmitResult {
        unimplemented!()
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

    async fn get_tasks(&self, key: &str) -> StorageResult<Vec<Task>> {
        unimplemented!()
    }
}

impl RedisStorage {
    async fn update() {
        unimplemented!()
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
