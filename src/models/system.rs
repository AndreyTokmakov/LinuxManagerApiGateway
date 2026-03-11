use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub uptime: String,
    pub architecture: String,
    pub kernel_version: String,
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub total_memory: String,
    pub load_average: String,
    pub boot_time: String,
    pub users_logged_in: u32,
}