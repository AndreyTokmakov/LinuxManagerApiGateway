use actix_web::{get, web, HttpResponse, Responder};
use tokio::join;
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/system",
    responses((status = 200, description = "System info", body = SystemInfo))
)]
#[get("/system")]
pub async fn system_info(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let (hostname_res, uptime_res, os_res) = join!(
        runner.execCommand("hostname", false),
        runner.execCommand("uptime -p", false),
        runner.execCommand("uname -a", false)
    );

    let hostname: String = hostname_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let uptime: String = uptime_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let os: String = os_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();

    HttpResponse::Ok().json(SystemInfo { hostname, uptime, os })
}

#[utoipa::path(
    get,
    path = "/system/memory",
    responses((status = 200, description = "Memory info", body = MemoryInfo))
)]
#[get("/system/memory")]
pub async fn memory_info(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let output: String = runner.execCommand(
        "free -h", false
    ).await.map(|r| r.stdout).unwrap_or_default();

    let mut total: String  = String::new();
    let mut used: String  = String::new();
    let mut free: String  = String::new();

    for line in output.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                total = parts[1].to_string();
                used = parts[2].to_string();
                free = parts[3].to_string();
            }
        }
    }

    HttpResponse::Ok().json(MemoryInfo { total, used, free })
}