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
    let output: String  = runner.execCommand(
        "df --output=source,fstype,size,used,avail,pcent,target", false
    ).await.map(|r| r.stdout).unwrap_or_default();

    let mut disks: Vec<DiskInfo> = Vec::new();
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 7 {
            let filesystem: String = parts[0].to_string();
            let fs_type: String  = parts[1].to_string();
            let size: String  = parts[2].to_string();
            let used: String  = parts[3].to_string();
            let avail: String  = parts[4].to_string();
            let used_percentage: String  = parts[5].to_string();
            let mount_point: String  = parts[6].to_string();

            // inode info
            let inode_out: String  = runner.execCommand(
                &format!("df -i {} | tail -1", mount_point),
                false
            ).await.map(|r| r.stdout).unwrap_or_default();

            let inode_parts: Vec<&str> = inode_out.split_whitespace().collect();
            let (inode_total, inode_used, inode_free) = if inode_parts.len() >= 4 {
                (
                    inode_parts[1].parse::<u64>().ok(),
                    inode_parts[2].parse::<u64>().ok(),
                    inode_parts[3].parse::<u64>().ok()
                )
            } else {
                (None, None, None)
            };

            // UUID и device через blkid
            let blkid_out = runner.execCommand(
                &format!("blkid -s UUID -o value {}", filesystem),
                false
            ).await.ok().map(|r| r.stdout.trim().to_string());

            // device name (например /dev/sda1)
            let device_name = Some(filesystem.clone());

            disks.push(DiskInfo {
                filesystem,
                fs_type,
                size,
                used,
                avail,
                used_percentage,
                mount_point,
                inode_total,
                inode_used,
                inode_free,
                uuid: blkid_out,
                device: device_name,
            });
        }
    }

    HttpResponse::Ok().json(disks)
}