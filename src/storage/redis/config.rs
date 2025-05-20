use getset::{CopyGetters, Getters};
use serde::{Deserialize};

#[derive(Deserialize, CopyGetters, Getters, Clone)]
#[getset(get = "pub")]
pub struct RedisConfig {
    host: String,
    port: u64,
    // #[getset(skip)]
    // #[getset(get_copy = "pub")]
    // expired: u64,
}

// fn test(){
//     let redis = RedisConfig{
//     address: "122".to_string(),
//     expired: 3600
//     };
//     redis.address()
// }