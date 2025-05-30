use redis::{ErrorKind, RedisError};
use std::error::Error;

use crate::storage::error::StorageError;

impl From<RedisError> for StorageError {
    fn from(err: RedisError) -> Self {
        match err.kind() {
            ErrorKind::ResponseError | ErrorKind::ParseError | ErrorKind::AuthenticationFailed => {
                StorageError::ServiceUnavailable("Redis".to_owned())
            }

            ErrorKind::TypeError | ErrorKind::ExecAbortError | ErrorKind::BusyLoadingError => {
                StorageError::ClientNotFound("client not found".to_owned())
            }

            // ErrorKind::NoScriptError => {}
            // ErrorKind::InvalidClientConfig => {}
            // ErrorKind::Moved => {}
            // ErrorKind::Ask => {}
            // ErrorKind::TryAgain => {}
            // ErrorKind::ClusterDown => {}
            // ErrorKind::CrossSlot => {}
            // ErrorKind::MasterDown => {}
            // ErrorKind::IoError => {}
            // ErrorKind::ClientError => {}
            // ErrorKind::ExtensionError => {}
            // ErrorKind::ReadOnly => {}
            // ErrorKind::MasterNameNotFoundBySentinel => {}
            // ErrorKind::NoValidReplicasFoundBySentinel => {}
            // ErrorKind::EmptySentinelList => {}
            // ErrorKind::NotBusy => {}
            // ErrorKind::ClusterConnectionNotFound => {}
            // ErrorKind::NoSub => {}
            // ErrorKind::RESP3NotSupported => {}
            ErrorKind::Serialize => StorageError::KeyNotFound("Key not found!".to_owned()),
            _ => StorageError::AnotherError(err.to_string()),
        }
    }
}
