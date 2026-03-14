use actix_web::{get, web, HttpResponse, Responder};
use std::collections::HashMap;

use crate::models::MemoryInfo;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/memory",
    responses(
        (status = 200, description = "Memory information", body = MemoryInfo)
    )
)]
#[get("/memory")]
pub async fn memory_info(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let output: String = runner.execCommand(
        "cat /proc/meminfo", false
    ).await.map(|r| r.stdout).unwrap_or_default();

    let mut map: HashMap<String, u64> = HashMap::new();
    for line in output.lines()
    {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2
        {
            let key = parts[0].trim_end_matches(':').to_string();
            if let Ok(value) = parts[1].parse::<u64>() {
                map.insert(key, value);
            }
        }
    }

    let info = MemoryInfo
    {
        mem_total: *map.get("MemTotal").unwrap_or(&0),
        mem_free: *map.get("MemFree").unwrap_or(&0),
        mem_available: *map.get("MemAvailable").unwrap_or(&0),
        buffers: *map.get("Buffers").unwrap_or(&0),
        cached: *map.get("Cached").unwrap_or(&0),
        active: *map.get("Active").unwrap_or(&0),
        inactive: *map.get("Inactive").unwrap_or(&0),
        swap_total: *map.get("SwapTotal").unwrap_or(&0),
        swap_free: *map.get("SwapFree").unwrap_or(&0),
        slab: *map.get("Slab").unwrap_or(&0),
        dirty: *map.get("Dirty").unwrap_or(&0),
        anon_pages: *map.get("AnonPages").unwrap_or(&0),
    };

    HttpResponse::Ok().json(info)
}