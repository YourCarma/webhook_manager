use redis::{ErrorKind, RedisError, ServerErrorKind};

use crate::storage::error::StorageError;

impl From<RedisError> for StorageError {
    fn from(err: RedisError) -> Self {
        match err.kind() {
            ErrorKind::AuthenticationFailed
            | ErrorKind::Io
            | ErrorKind::Client
            | ErrorKind::ClusterConnectionNotFound => {
                StorageError::ServiceUnavailable("Redis".to_owned())
            }

            ErrorKind::UnexpectedReturnType
            | ErrorKind::Server(ServerErrorKind::ExecAbort)
            | ErrorKind::Server(ServerErrorKind::BusyLoading) => {
                StorageError::ClientNotFound("client not found".to_owned())
            }

            ErrorKind::Serialize => StorageError::KeyNotFound("Key not found!".to_owned()),
            _ => StorageError::AnotherError(err.to_string()),
        }
    }
}
