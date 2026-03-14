use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ProcessInfo
{
    pub pid: u32,
    pub ppid: u32,
    pub state: String,
    pub name: String,
    pub cpu: f64,
    pub mem: f64,
    pub rss_kb: u64,
    pub vsize_kb: u64,
    pub threads: u32,
    pub start_time: u64,
    pub cmd: String
}