use serde::Serialize;
use utoipa::ToSchema;

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