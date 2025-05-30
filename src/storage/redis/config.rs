use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Deserialize, CopyGetters, Getters, Clone)]
#[getset(get = "pub")]
pub struct RedisConfig {
    address: String,
    #[getset(skip)]
    #[getset(get_copy = "pub")]
    expired: u64,
}
