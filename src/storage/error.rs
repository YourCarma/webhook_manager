use core::error;

use redis::RedisError;
use thiserror::Error;

pub type StorageResult<T> = Result<T, StorageError>;


#[derive(Debug, Error)]
pub enum StorageError{
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Client not found: {0}")]
    ClientNotFound(String),
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Redis error: {0}")]
    RedisError(#[from]RedisError),
    #[error("Key {0} not found")]
    KeyNotFound(String)
}
