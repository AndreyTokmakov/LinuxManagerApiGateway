use actix_web::{get, post, web, HttpResponse, Responder};
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/services",
    responses((status = 200, description = "Service status", body = Vec<ServiceStatus>))
)]
#[get("/services")]
pub async fn services_status(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let output: String = runner.execCommand(
        "systemctl list-units --type=service --no-pager --no-legend", false
    ).await.map(|r| r.stdout).unwrap_or_default();

    let mut services: Vec<ServiceStatus> = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {

            let name = parts[0];

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
                }
                else if let Some(value) = dline.strip_prefix("LoadState=") {
                    load_state = value.to_string();
                }
                else if let Some(value) = dline.strip_prefix("ActiveState=") {
                    active_state = value.to_string();
                }
                else if let Some(value) = dline.strip_prefix("SubState=") {
                    sub_state = value.to_string();
                }
                else if let Some(value) = dline.strip_prefix("Type=") {
                    service_type = value.to_string();
                }
                else if let Some(value) = dline.strip_prefix("ExecMainPID=") {
                    if let Ok(pid) = value.parse::<u32>() {
                        main_pid = Some(pid);
                    }
                }
                else if let Some(value) = dline.strip_prefix("MemoryCurrent=") {
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


#[utoipa::path(
    get,
    path = "/services/{name}",
    params(
        ("name" = String, Path, description = "systemd service name")
    ),
    responses(
        (status = 200, description = "Service details", body = ServiceDetails)
    )
)]
#[get("/services/{name}")]
pub async fn service_details(
    runner: web::Data<SshCommandRunner>,
    path: web::Path<String>,
) -> impl Responder {

    let service = path.into_inner();

    let cmd = format!(
        "systemctl show {} \
        --property=Description,LoadState,ActiveState,SubState,Type,\
ExecMainPID,MemoryCurrent,NRestarts,ExecStart,ExecStop",
        service
    );

    let output = runner.execCommand(&cmd, false)
        .await
        .map(|r| r.stdout)
        .unwrap_or_default();

    let mut description = String::new();
    let mut load_state = String::new();
    let mut active_state = String::new();
    let mut sub_state = String::new();
    let mut service_type = String::new();
    let mut main_pid = None;
    let mut memory_current = None;
    let mut restart_count = None;
    let mut exec_start = None;
    let mut exec_stop = None;

    for line in output.lines() {

        if let Some(v) = line.strip_prefix("Description=") {
            description = v.to_string();
        }
        else if let Some(v) = line.strip_prefix("LoadState=") {
            load_state = v.to_string();
        }
        else if let Some(v) = line.strip_prefix("ActiveState=") {
            active_state = v.to_string();
        }
        else if let Some(v) = line.strip_prefix("SubState=") {
            sub_state = v.to_string();
        }
        else if let Some(v) = line.strip_prefix("Type=") {
            service_type = v.to_string();
        }
        else if let Some(v) = line.strip_prefix("ExecMainPID=") {
            main_pid = v.parse::<u32>().ok();
        }
        else if let Some(v) = line.strip_prefix("MemoryCurrent=") {
            memory_current = Some(v.to_string());
        }
        else if let Some(v) = line.strip_prefix("NRestarts=") {
            restart_count = v.parse::<u32>().ok();
        }
        else if let Some(v) = line.strip_prefix("ExecStart=") {
            exec_start = Some(v.to_string());
        }
        else if let Some(v) = line.strip_prefix("ExecStop=") {
            exec_stop = Some(v.to_string());
        }
    }

    HttpResponse::Ok().json(ServiceDetails {
        name: service,
        description,
        load_state,
        active_state,
        sub_state,
        service_type,
        main_pid,
        memory_current,
        restart_count,
        exec_start,
        exec_stop
    })
}

#[utoipa::path(
    post,
    path = "/services/{name}/{action}",
    params(
        ("name" = String, Path, description = "systemd service name"),
        ("action" = ServiceAction, Path, description = "Action to perform on the service")
    ),
    responses(
        (status = 200, description = "Service action result"),
        (status = 400, description = "Invalid action")
    )
)]
#[post("/services/{name}/{action}")]
pub async fn service_action(runner: web::Data<SshCommandRunner>,
                            path: web::Path<(String, ServiceAction)>) -> impl Responder
{
    let (service_name, action) = path.into_inner();
    let cmd = match action {
        ServiceAction::Start => "start",
        ServiceAction::Stop => "stop",
        ServiceAction::Restart => "restart",
        ServiceAction::Enable => "enable",
        ServiceAction::Disable => "disable",
    };

    let ssh_cmd = format!("sudo systemctl {} {}", cmd, service_name);
    match runner.execCommand(&ssh_cmd, true).await {
        Ok(res) => HttpResponse::Ok().body(format!("{} executed, exit code {}", cmd, res.exitCode)),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string())
    }
}

#[utoipa::path(
    get,
    path = "/services/{name}/logs",
    params(
        ("name" = String, Path, description = "systemd service name")
    ),
    responses(
        (status = 200, description = "Service logs")
    )
)]
#[get("/services/{name}/logs")]
pub async fn service_logs(runner: web::Data<SshCommandRunner>,
                          path: web::Path<String>) -> impl Responder
{
    let name: String = path.into_inner();
    let cmd: String = format!("journalctl -u {} --no-pager -n 100", name);
    let output: String = runner.execCommand(&cmd, true)
        .await.map(|r| r.stdout).unwrap_or_default();
    HttpResponse::Ok().body(output)
}