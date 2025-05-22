use getset::{Getters, Setters};
use serde::{de::Error, Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use redis::{RedisError, RedisResult, RedisWrite, Value};


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus{
    Pending,
    Awaiting,
    Processing,
    Ready,
    Error,
}

#[derive(Serialize, Deserialize, Getters, Setters)]
pub struct Task{
    task_id: Uuid,
    user_id: Uuid,
    #[getset(set = "pub")]
    progress: TaskProgress,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[getset(set = "pub")]
    response_data: String,
}

impl redis::ToRedisArgs for Task {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        match serde_json::to_string(self) {
            Ok(json_str) => out.write_arg_fmt(json_str),
            Err(err) => {
                tracing::error!(err=?err, "REDIS: failed to serialize TaskForm");
            }
        }
    }
}

impl redis::FromRedisValue for Task {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        match v {
            Value::BulkString(data) => {
                serde_json::from_slice::<Task>(data.as_slice()).map_err(RedisError::from)
            }
            _ => {
                let err = serde_json::Error::custom("failed to extract redis value type");
                Err(RedisError::from(err))
            }
        }
    }
}

#[derive(Serialize, Deserialize,)]
pub struct  TaskProgress {
    status: TaskStatus,
    progress: f32,
}


