use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct MemoryInfo
{
    pub total: String,
    pub used: String,
    pub free: String,
    pub available: String,
    pub swap_total: String,
    pub swap_used: String,
    pub swap_free: String,
}