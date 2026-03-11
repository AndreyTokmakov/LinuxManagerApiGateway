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
    let output: String = runner.execCommand(
        "systemctl list-units --type=service --no-pager --no-legend", false
    ).await.map(|r| r.stdout).unwrap_or_default();

    let mut services: Vec<ServiceStatus> = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let name = parts[0];
            // Для каждого сервиса получаем подробности
            let details = runner.execCommand(
                &format!("systemctl show {} --no-page --property=Description,LoadState,ActiveState,SubState,Type,ExecMainPID,MemoryCurrent", name),
                false
            ).await
                .map(|r| r.stdout)
                .unwrap_or_default();

            let mut description = String::new();
            let mut load_state = String::new();
            let mut active_state = String::new();
            let mut sub_state = String::new();
            let mut service_type = String::new();
            let mut main_pid = None;
            let mut memory_current = None;

            for dline in details.lines() {
                if let Some(value) = dline.strip_prefix("Description=") {
                    description = value.to_string();
                } else if let Some(value) = dline.strip_prefix("LoadState=") {
                    load_state = value.to_string();
                } else if let Some(value) = dline.strip_prefix("ActiveState=") {
                    active_state = value.to_string();
                } else if let Some(value) = dline.strip_prefix("SubState=") {
                    sub_state = value.to_string();
                } else if let Some(value) = dline.strip_prefix("Type=") {
                    service_type = value.to_string();
                } else if let Some(value) = dline.strip_prefix("ExecMainPID=") {
                    if let Ok(pid) = value.parse::<u32>() {
                        main_pid = Some(pid);
                    }
                } else if let Some(value) = dline.strip_prefix("MemoryCurrent=") {
                    if !value.is_empty() {
                        memory_current = Some(value.to_string());
                    }
                }
            }

            services.push(ServiceStatus {
                name: name.to_string(),
                description,
                active: active_state == "active",
                sub: sub_state,
                load_state,
                service_type,
                main_pid,
                memory_current,
            });
        }
    }

    HttpResponse::Ok().json(services)
}