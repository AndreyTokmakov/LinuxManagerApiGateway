use actix_web::{get, web, HttpResponse, Responder};
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/services",
    responses((status = 200, description = "Service status", body = Vec<ServiceStatus>))
)]
#[get("/services")]
pub async fn services_status(runner: web::Data<SshCommandRunner>) -> impl Responder {
    let output = runner.execCommand(
        "systemctl list-units --type=service --no-pager --no-legend",
        false
    ).await
        .map(|r| r.stdout)
        .unwrap_or_default();

    let mut services = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            services.push(ServiceStatus {
                name: parts[0].to_string(),
                active: parts[2] == "active",
            });
        }
    }

    HttpResponse::Ok().json(services)
}