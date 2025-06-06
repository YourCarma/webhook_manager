use getset::Getters;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::storage::models::{Task, TaskProgress};

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct TaskCreation {
    #[schema(
        example = "384f4d80-4ed6-4032-8569-f02fd5e1afb9:service_name:384f4d80-4ed6-4032-2569-f02fd5e1afb9"
    )]
    key: String,
    task: Task,
}

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct ProgressUpdate {
    #[schema(
        example = "384f4d80-4ed6-4032-8569-f02fd5e1afb9:service_name:384f4d80-4ed6-4032-2569-f02fd5e1afb9"
    )]
    key: String,
    progress: TaskProgress,
}

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct ResponseDataUpdate {
    #[schema(
        example = "384f4d80-4ed6-4032-8569-f02fd5e1afb9:service_name:384f4d80-4ed6-4032-2569-f02fd5e1afb9"
    )]
    key: String,
    response_data: String,
}

#[derive(Serialize, Deserialize, Getters, ToSchema)]
#[getset(get = "pub")]
pub struct TaskID {
    #[schema(
        example = "384f4d80-4ed6-4032-8569-f02fd5e1afb9:service_name:384f4d80-4ed6-4032-2569-f02fd5e1afb9"
    )]
    key: String,
}
