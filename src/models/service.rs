use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
#[schema(example = "start")]
pub enum ServiceAction
{
    Start,
    Stop,
    Restart,
    Enable,
    Disable,
}

#[derive(Serialize, ToSchema)]
pub struct ServiceStatus {
    pub name: String,
    pub description: String,
    pub active: bool,
    pub sub: String,
    pub load_state: String,
    pub service_type: String,
    pub main_pid: Option<u32>,
    pub memory_current: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ServiceDetails
{
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub service_type: String,
    pub main_pid: Option<u32>,
    pub memory_current: Option<String>,
    pub restart_count: Option<u32>,
    pub exec_start: Option<String>,
    pub exec_stop: Option<String>,
}