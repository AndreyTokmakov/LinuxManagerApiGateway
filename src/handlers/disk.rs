use actix_web::{get, web, HttpResponse, Responder};
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/disk",
    responses((status = 200, description = "Disk info", body = Vec<DiskInfo>))
)]
#[get("/disk")]
pub async fn disk_info(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let output: String = runner.execCommand(
        "df -h --output=source,size,used,avail,target -x tmpfs -x devtmpfs", false
    ).await.map(|r| r.stdout).unwrap_or_default();

    let mut disks: Vec<DiskInfo> = Vec::new();
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