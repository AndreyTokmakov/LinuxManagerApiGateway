use actix_web::{get, web, HttpResponse, Responder};
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/network/interfaces",
    responses((status = 200, description = "Network interfaces", body = Vec<NetworkInterface>))
)]
#[get("/network/interfaces")]
pub async fn interfaces(runner: web::Data<SshCommandRunner>) -> impl Responder {
    let output = runner.execCommand("ip -o -4 addr show", false)
        .await
        .map(|r| r.stdout)
        .unwrap_or_default();

    let mut list = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            list.push(NetworkInterface {
                name: parts[1].to_string(),
                address: parts[3].to_string(),
            });
        }
    }

    HttpResponse::Ok().json(list)
}