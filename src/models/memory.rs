use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct MemoryInfo
{
    pub mem_total: u64,
    pub mem_free: u64,
    pub mem_available: u64,
    pub buffers: u64,
    pub cached: u64,
    pub active: u64,
    pub inactive: u64,
    pub swap_total: u64,
    pub swap_free: u64,
    pub slab: u64,
    pub dirty: u64,
    pub anon_pages: u64,
}