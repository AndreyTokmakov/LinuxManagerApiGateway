use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CommandRequest {
    pub command: String,
    pub sudo: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct CommandResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}