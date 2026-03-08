use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ServiceStatus {
    pub name: String,
    pub active: bool,
}