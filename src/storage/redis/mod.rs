pub mod config;
mod error;

use std::sync::Arc;

use redis::{RedisError, Client, AsyncCommands, ToRedisArgs, FromRedisValue, RedisResult};
use getset::{Getters, Setters, CopyGetters};
use tokio::{sync::RwLock};

use crate::storage::redis::config::RedisConfig;
use crate::storage::TaskStorage;
use crate::ServiceConnect;
use crate::storage::error::StorageError;
use super::{error::StorageResult, models::{Task, TaskProgress}};




#[derive(Clone, CopyGetters)]
pub struct RedisClient {
    options: Arc<RedisConfig>,
    client: Arc<RwLock<Client>>,
}

#[async_trait::async_trait]
impl ServiceConnect for RedisClient {
    type Config = RedisConfig;
    type Error = RedisError;
    type Client = RedisClient;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        let address = format!("redis://{}:{}", config.host().as_str(), config.port());
        let client = Client::open(address)?;
        Ok(RedisClient {
            options: Arc::new(config.to_owned()),
            client: Arc::new(RwLock::new(client)),
        })
    }
}


#[async_trait::async_trait]
impl TaskStorage for RedisClient
{

    async fn get_task(&self, key: &str) -> Option<Task>  {
        let cxt = self.client.read().await;
        match cxt.get_multiplexed_tokio_connection().await {
            Ok(mut conn) => { 
                let result = conn.get(key).await;
                match result {
                    Ok(res) => {
                        res
                    },
                    Err(err) => {
                        tracing::warn!(err=?err, "failed to get redis key");
                        None
                    }
                }
            },
            Err(err) => {
                tracing::warn!(err=?err, "failed to get redis service connection");
                None
            }
        }

    }

    async fn create_task(&self, key: &str, value: Task) -> StorageResult<()>{

        let expired_secs = self.options.expired();
        let cxt = self.client.write().await;
        match cxt.get_multiplexed_tokio_connection().await {
            Err(err) => {
                tracing::warn!(err=?err, "failed to get redis service connection");
                return Err(crate::storage::error::StorageError::ServiceUnavailable("Service is not available".to_string()));
            }
            Ok(mut conn) => {
                let set_result: RedisResult<()> = conn.set_ex(key, value, expired_secs).await;
                if let Err(err) = set_result {
                    tracing::error!(err=?err, "failed to insert value to redis");
                    return  Err(StorageError::RedisError(err));
                }
            }
        };

    }

    async fn update_progress(&self, key: &str, value: TaskProgress) -> StorageResult<()>{
        let cxt = self.client.write().await;
        match cxt.get_multiplexed_tokio_connection().await {
            Err(err) => {
                tracing::warn!(err=?err, "failed to get redis service connection");
                return  Err(StorageError::ServiceUnavailable("Redis".to_string()))
            }
            Ok(mut conn) => {
                let key_exists: bool = match conn.exists(&key).await {
                Ok(exists) => { 
                    exists 
                }
                Err(err) => {
                    tracing::error!(err=?err, "Failed to check an existing value");
                    return Err(StorageError::RedisError(err));
                }
                };
                if !key_exists{
                    return  Err(StorageError::KeyNotFound(key.to_string()));
                };
                let mut task: Task = match conn.get(&key).await{
                    Ok(task) => { 
                    task 
                        }
                    Err(err) => {
                        tracing::error!(err=?err, "Failed to check an existing value");
                        return Err(StorageError::RedisError(err));
                        }
                    };
                task.set_progress(value);
                let result: RedisResult<()> = conn.set(&key, task).await;
                match result {
                     Ok(task) => { 
                    return Ok(());
                        }
                    Err(err) => {
                        tracing::error!(err=?err, "Failed to update an progress");
                        return Err(StorageError::RedisError(err));
                        }
                };
            }
        }

    }

    async fn add_response_data(&self, key: &str, data: String) {
        unimplemented!()
    }

    async fn get_client_tasks(&self, key: &str){
        unimplemented!()
    }
}


#[test]
fn test_add() { 
 
    let input_1 = 2;
    let input_2 = 8;
    let result = add(input_1, input_2);
    assert_eq!(result, 10, "The addition result is incorrect.");
}



