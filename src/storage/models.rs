use std::str::FromStr;

use chrono::{DateTime, Utc};
use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use redis::{ParsingError, RedisWrite, Value};

fn current_timestamp() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    #[default]
    Pending,
    Awaiting,
    Processing,
    Ready,
    Error,
}

#[derive(Serialize, Deserialize, Getters, Setters, PartialEq, Debug, Clone, ToSchema)]
#[getset(get = "pub", set = "pub")]
pub struct Task {
    #[schema(value_type = String, format = "uuid", example = "384f4d80-4ed6-4032-2569-f02fd5e1afb9")]
    task_id: Uuid,
    #[schema(default = "guest", example = "guest")]
    user_id: String,
    #[getset(set = "pub")]
    #[schema(default = "general", example = "general")]
    service: String,
    #[getset(set = "pub")]
    progress: TaskProgress,
    #[serde(default = "current_timestamp")]
    #[schema(example = "2025-07-09T12:51:27.948Z")]
    created_at: DateTime<Utc>,
    #[getset(set = "pub")]
    #[serde(default = "current_timestamp")]
    #[schema(example = "2025-07-09T12:55:27.948Z")]
    updated_at: DateTime<Utc>,
    #[getset(set = "pub")]
    #[schema(example = "{\"result\":\"ok\"}")]
    response_data: String,
}

#[derive(Serialize, Deserialize, Getters, Setters, Debug, Clone, ToSchema)]
#[getset(get = "pub", set = "pub")]
pub struct FormattedTask {
    #[serde(flatten)]
    task: Task,
    expire: i64,
}

#[derive(Serialize, Deserialize, Getters, Setters, Default, PartialEq, Debug, Clone, ToSchema)]
#[getset(get = "pub", set = "pub")]
pub struct TaskProgress {
    #[schema(example = "PROCESSING")]
    status: TaskStatus,
    #[schema(example = 0.5)]
    progress: f32,
}

impl Default for Task {
    fn default() -> Self {
        let datetime = DateTime::parse_from_rfc3339("2025-05-26T14:18:48.717056300Z")
            .unwrap()
            .with_timezone(&Utc);
        Self {
            task_id: Uuid::from_str("96366fb0-0c0f-4671-8f3f-8a98641d11ae").unwrap(),
            user_id: "guest".to_owned(),
            service: "general".to_owned(),
            progress: TaskProgress::default(),
            created_at: datetime,
            updated_at: datetime,
            response_data: String::new(),
        }
    }
}

impl Default for FormattedTask {
    fn default() -> Self {
        Self {
            expire: 1800,
            task: Task::default(),
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

impl redis::ToSingleRedisArg for Task {}

impl redis::FromRedisValue for Task {
    fn from_redis_value(v: Value) -> Result<Self, ParsingError> {
        match v {
            Value::BulkString(data) => serde_json::from_slice::<Task>(data.as_slice())
                .map_err(|err| ParsingError::from(err.to_string())),
            _ => Err(ParsingError::from("failed to extract redis value type")),
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

impl redis::ToSingleRedisArg for TaskProgress {}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::Task;

    #[test]
    fn deserializes_missing_timestamps_with_current_time() {
        let payload = r#"{
            "task_id": "96366fb0-0c0f-4671-8f3f-8a98641d11ae",
            "user_id": "guest",
            "service": "general",
            "progress": {
                "status": "PENDING",
                "progress": 0.0
            },
            "response_data": ""
        }"#;

        let before = Utc::now();
        let task: Task = serde_json::from_str(payload).unwrap();
        let after = Utc::now();

        assert!(*task.created_at() >= before);
        assert!(*task.created_at() <= after);
        assert!(*task.updated_at() >= before);
        assert!(*task.updated_at() <= after);
    }
}
