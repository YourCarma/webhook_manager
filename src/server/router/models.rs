use getset::Getters;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::storage::models::{Task, TaskProgress};

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct TaskCreation {
    #[schema(example = "guest:general:384f4d80-4ed6-4032-2569-f02fd5e1afb9")]
    key: String,
    task: Task,
}

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct TaskCreationV2 {
    task: Task,
}

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct ProgressUpdate {
    #[schema(example = "guest:general:384f4d80-4ed6-4032-2569-f02fd5e1afb9")]
    key: String,
    progress: TaskProgress,
}

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct ResponseDataUpdate {
    #[schema(example = "guest:general:384f4d80-4ed6-4032-2569-f02fd5e1afb9")]
    key: String,
    response_data: String,
}

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct TaskID {
    #[schema(example = "guest:general:384f4d80-4ed6-4032-2569-f02fd5e1afb9")]
    key: String,
}

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct TaskListQuery {
    #[schema(example = "guest")]
    user_id: Option<String>,
}
