use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct DiskInfo {
    pub filesystem: String,
    pub mount_point: String,
    pub size: String,
    pub used: String,
    pub avail: String,
    pub used_percentage: String,
    pub fs_type: String,
    pub inode_total: Option<u64>,
    pub inode_used: Option<u64>,
    pub inode_free: Option<u64>,
    pub uuid: Option<String>,
    pub device: Option<String>,
}