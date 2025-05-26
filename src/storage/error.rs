use thiserror::Error;

pub type StorageResult<T> = Result<T, StorageError>;
pub type SubmitResult = Result<(), StorageError>;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Client not found: {0}")]
    ClientNotFound(String),
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Another Error: {0}")]
    AnotherError(String),
    #[error("Key {0} not found")]
    KeyNotFound(String),
}
