use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SystemInfo {
    pub hostname: String,
    pub uptime: String,
    pub os: String,
}

#[derive(Serialize, ToSchema)]
pub struct MemoryInfo {
    pub total: String,
    pub used: String,
    pub free: String,
}