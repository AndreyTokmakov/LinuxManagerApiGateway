use actix_web::{get, web, HttpResponse, Responder};

use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/memory",
    responses((status = 200, description = "Memory info", body = MemoryInfo))
)]
#[get("/memory")]
pub async fn memory_info(
    runner: web::Data<SshCommandRunner>
) -> impl Responder
{
    let output: String = runner.execCommand(
        "free -h", false
    ).await.map(|r| r.stdout).unwrap_or_default();

    let mut total = String::new();
    let mut used = String::new();
    let mut free = String::new();
    let mut available = String::new();

    let mut swap_total = String::new();
    let mut swap_used = String::new();
    let mut swap_free = String::new();

    for line in output.lines()
    {
        if line.starts_with("Mem:")
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7
            {
                total = parts[1].to_string();
                used = parts[2].to_string();
                free = parts[3].to_string();
                available = parts[6].to_string();
            }
        }
        if line.starts_with("Swap:")
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4
            {
                swap_total = parts[1].to_string();
                swap_used = parts[2].to_string();
                swap_free = parts[3].to_string();
            }
        }
    }

    HttpResponse::Ok().json(
        MemoryInfo {
            total,
            used,
            free,
            available,
            swap_total,
            swap_used,
            swap_free
        }
    )
}