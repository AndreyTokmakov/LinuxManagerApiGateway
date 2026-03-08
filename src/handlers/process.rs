use actix_web::{get, web, HttpResponse, Responder};
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/processes",
    responses((status = 200, description = "Processes", body = Vec<ProcessInfo>))
)]
#[get("/processes")]
pub async fn processes(runner: web::Data<SshCommandRunner>) -> impl Responder {
    let output = runner.execCommand("ps -eo pid,pcpu,pmem,cmd --no-headers", false)
        .await
        .map(|r| r.stdout)
        .unwrap_or_default();

    let mut list = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            list.push(ProcessInfo {
                pid: parts[0].parse().unwrap_or(0),
                cpu: parts[1].to_string(),
                mem: parts[2].to_string(),
                command: parts[3..].join(" "),
            });
        }
    }

    HttpResponse::Ok().json(list)
}