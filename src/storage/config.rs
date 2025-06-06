use getset::{CopyGetters, Getters};
use serde::Deserialize;

use crate::storage::redis::config::RedisConfig;


#[derive(Clone, Deserialize, CopyGetters, Getters)]
#[getset(get = "pub")]
pub struct StorageConfig {
    redis: RedisConfig,
}
