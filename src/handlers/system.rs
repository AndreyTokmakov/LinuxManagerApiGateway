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
    let (
        hostname_res,
        os_res,
        uptime_res,
        arch_res,
        kernel_res,
        cpu_model_res,
        cpu_cores_res,
        mem_res,
        load_res,
        boot_res,
        users_res
    ) = join!(
        runner.execCommand("hostname", false),
        runner.execCommand("uname -s", false),
        runner.execCommand("uptime -p", false),
        runner.execCommand("uname -m", false),
        runner.execCommand("uname -r", false),
        runner.execCommand("lscpu | grep 'Model name' | awk -F ':' '{print $2}'", false),
        runner.execCommand("nproc", false),
        runner.execCommand("free -h | grep Mem: | awk '{print $2}'", false),
        runner.execCommand("uptime | awk -F 'load average:' '{print $2}'", false),
        runner.execCommand("who -b | awk '{print $3 \" \" $4}'", false),
        runner.execCommand("who | wc -l", false)
    );

    let hostname = hostname_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let os = os_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let uptime = uptime_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let architecture = arch_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let kernel_version = kernel_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let cpu_model = cpu_model_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let cpu_cores = cpu_cores_res.map(|r| r.stdout.trim().parse::<u32>().unwrap_or(0)).unwrap_or(0);
    let total_memory = mem_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let load_average = load_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let boot_time = boot_res.map(|r| r.stdout.trim().to_string()).unwrap_or_default();
    let users_logged_in = users_res.map(|r| r.stdout.trim().parse::<u32>().unwrap_or(0)).unwrap_or(0);

    HttpResponse::Ok().json(SystemInfo {
        hostname,
        os,
        uptime,
        architecture,
        kernel_version,
        cpu_model,
        cpu_cores,
        total_memory,
        load_average,
        boot_time,
        users_logged_in,
    })
}