use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ProcessInfo {
    pub pid: u32,
    pub cpu: String,
    pub mem: String,
    pub command: String,
}