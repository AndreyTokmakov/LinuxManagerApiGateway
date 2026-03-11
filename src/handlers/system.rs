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