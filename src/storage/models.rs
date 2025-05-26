use std::str::FromStr;

use chrono::{DateTime, Utc};
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize, de::Error};
use uuid::Uuid;

use redis::{RedisError, RedisResult, RedisWrite, Value};

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    #[default]
    Pending,
    Awaiting,
    Processing,
    Ready,
    Error,
}

#[derive(Serialize, Deserialize, Getters, Setters, PartialEq, Debug, Clone)]
#[getset(get = "pub", set = "pub")]
pub struct Task {
    task_id: Uuid,
    user_id: Uuid,
    #[getset(set = "pub")]
    progress: TaskProgress,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[getset(set = "pub")]
    response_data: String,
}

#[derive(Serialize, Deserialize, Getters, Setters, Debug, Clone)]
#[getset(get = "pub", set = "pub")]
pub struct FormattedTask {
    #[serde(flatten)]
    task: Task,
    expire: u64,
}

#[derive(Serialize, Deserialize, Getters, Setters, Default, PartialEq, Debug, Clone)]
#[getset(get = "pub", set = "pub")]
pub struct TaskProgress {
    status: TaskStatus,
    progress: f32,
}

impl Default for Task {
    fn default() -> Self {
        let datetime = DateTime::parse_from_rfc3339("2025-05-26T14:18:48.717056300Z").unwrap().with_timezone(&Utc);
        Self {
            task_id: Uuid::from_str("96366fb0-0c0f-4671-8f3f-8a98641d11ae").unwrap(),  
            user_id: Uuid::from_str("96366fb0-0c0f-4671-8f3f-8a98641d11ae").unwrap(),     
            progress: TaskProgress::default(),
            created_at: datetime,   
            updated_at: datetime,
            response_data: String::new(),
        }
    }
}

impl  Default for FormattedTask {
    fn default() -> Self {
        Self {
            expire: 1800,
            task: Task::default()
        }
    }
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

impl redis::ToRedisArgs for TaskProgress {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        match serde_json::to_string(self) {
            Ok(json_str) => out.write_arg_fmt(json_str),
            Err(err) => {
                tracing::error!(err=?err, "REDIS: failed to serialize TaskProgress Form");
            }
        }
    }
}


