
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tokio::join;

#[utoipa::path(
    get,
    path = "/system",
    responses((status = 200, description = "System info", body = SystemInfo))
)]
#[get("/system")]
pub async fn system_info(runner: web::Data<SshCommandRunner>) -> impl Responder {
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

#[utoipa::path(
    get,
    path = "/system/memory",
    responses((status = 200, description = "Memory info", body = MemoryInfo))
)]
#[get("/system/memory")]
pub async fn memory_info(runner: web::Data<SshCommandRunner>) -> impl Responder {
    let output = runner.execCommand("free -h", false)
        .await
        .map(|r| r.stdout)
        .unwrap_or_default();

    let mut total = String::new();
    let mut used = String::new();
    let mut free = String::new();

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

#[utoipa::path(
    get,
    path = "/disk",
    responses((status = 200, description = "Disk info", body = Vec<DiskInfo>))
)]
#[get("/disk")]
pub async fn disk_info(runner: web::Data<SshCommandRunner>) -> impl Responder {
    let output = runner.execCommand(
        "df -h --output=source,size,used,avail,target -x tmpfs -x devtmpfs",
        false
    ).await
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

#[utoipa::path(
    get,
    path = "/services",
    responses((status = 200, description = "Service status", body = Vec<ServiceStatus>))
)]
#[get("/services")]
pub async fn services_status(runner: web::Data<SshCommandRunner>) -> impl Responder
{
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

#[utoipa::path(
    get,
    path = "/processes",
    responses((status = 200, description = "Processes", body = Vec<ProcessInfo>))
)]
#[get("/processes")]
pub async fn processes(runner: web::Data<SshCommandRunner>) -> impl Responder
{
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

#[utoipa::path(
    get,
    path = "/network/interfaces",
    responses((status = 200, description = "Network interfaces", body = Vec<NetworkInterface>))
)]
#[get("/network/interfaces")]
pub async fn interfaces(runner: web::Data<SshCommandRunner>) -> impl Responder
{
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

#[derive(OpenApi)]
#[openapi(
    paths(
        system_info,
        memory_info,
        disk_info,
        services_status,
        processes,
        interfaces
    ),
    components(
        schemas(
            SystemInfo,
            MemoryInfo,
            DiskInfo,
            ServiceStatus,
            ProcessInfo,
            NetworkInterface
        )
    )
)]
pub struct ApiDoc;

pub async fn run_server(host: &str, port: u16, runner: SshCommandRunner) -> std::io::Result<()> {
    let runner_data = web::Data::new(runner);
    println!("Starting server at http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(runner_data.clone())
            .service(system_info)
            .service(memory_info)
            .service(disk_info)
            .service(services_status)
            .service(processes)
            .service(interfaces)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
    })
        .bind((host, port))?
        .run()
        .await
}