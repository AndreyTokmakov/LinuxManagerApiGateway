use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ProcessInfo
{
    /// Process ID
    pub pid: u32,

    /// Parent process
    pub ppid: u32,

    /// Process state (R,S,D,Z,T)
    pub state: String,

    /// CPU usage %
    pub cpu: f64,

    /// Memory usage %
    pub mem: f64,

    /// Resident memory (KB)
    pub rss_kb: u64,

    /// Virtual memory size (KB)
    pub vsize_kb: u64,

    /// Thread count
    pub threads: u32,

    /// Process start time (ticks since boot)
    pub start_time: u64,

    /// Full command line
    pub cmd: String
}