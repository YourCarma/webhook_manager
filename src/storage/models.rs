use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus{
    Pending,
    Awaiting,
    Processing,
    Ready,
    Error,
}

#[derive(Serialize, Deserialize)]
pub struct Task{
    task_id: Uuid,
    user_id: Uuid,
    status: TaskStatus,
    progress: f32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    response_data: String,
}