use crate::ssh_connection_pool::ssh_connection_pool::{CommandResult, SshCommandRunner};

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use anyhow::Result;
use tokio::join;

#[derive(Serialize, utoipa::ToSchema)]
struct SystemInfo {
    hostname: String,
    uptime: String,
    os: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct DiskInfo {
    filesystem: String,
    size: String,
    used: String,
    avail: String,
    mount_point: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ServiceStatus {
    name: String,
    active: bool,
}

/// REST ручка для информации о системе
#[utoipa::path(
    get,
    path = "/system",
    responses(
        (status = 200, description = "System information", body = SystemInfo)
    )
)]
#[get("/system")]
async fn system_info(runner: web::Data<SshCommandRunner>) -> impl Responder {
    // Параллельно выполняем команды
    let (hostname_res, uptime_res, os_res) = join!(
        runner.execCommand("hostname", false),
        runner.execCommand("uptime -p", false),
        runner.execCommand("uname -a", false)
    );

    let hostname = hostname_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let uptime = uptime_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let os = os_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();

    HttpResponse::Ok().json(SystemInfo { hostname, uptime, os })
}

/// REST ручка для информации о дисках
#[utoipa::path(
    get,
    path = "/disk",
    responses(
        (status = 200, description = "Disk usage info", body = Vec<DiskInfo>)
    )
)]
#[get("/disk")]
async fn disk_info(runner: web::Data<SshCommandRunner>) -> impl Responder {
    let output = runner.execCommand("df -h --output=source,size,used,avail,target -x tmpfs -x devtmpfs", false)
        .await
        .map(|r| r.stdout)
        .unwrap_or_default();

    let mut disks = Vec::new();
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 5 {
            disks.push(DiskInfo {
                filesystem: parts[0].to_string(),
                size: parts[1].to_string(),
                used: parts[2].to_string(),
                avail: parts[3].to_string(),
                mount_point: parts[4].to_string(),
            });
        }
    }

    HttpResponse::Ok().json(disks)
}

/// REST ручка для информации о сервисах
#[utoipa::path(
    get,
    path = "/services",
    responses(
        (status = 200, description = "List of services", body = Vec<ServiceStatus>)
    )
)]
#[get("/services")]
async fn services_status(runner: web::Data<SshCommandRunner>) -> impl Responder {
    let output = runner.execCommand("systemctl list-units --type=service --no-pager --no-legend", false)
        .await
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

#[derive(OpenApi)]
#[openapi(
    paths(system_info, disk_info, services_status),
    components(schemas(SystemInfo, DiskInfo, ServiceStatus)),
    tags((name = "system", description = "System information API"))
)]
struct ApiDoc;

/// Функция запуска сервера, runner передаётся извне
pub async fn run_server(runner: SshCommandRunner, port: u16) -> std::io::Result<()>
{
    let runner_data = web::Data::new(runner);
    println!("Starting server on http://127.0.0.1:{}", port);

    HttpServer::new(move || { App::new()
        .app_data(runner_data.clone())
        .service(system_info)
        .service(disk_info)
        .service(services_status)
        .service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()), )
    }).bind(("127.0.0.1", port))?.run().await
}

// http://0.0.0.0:52525/swagger-ui/
