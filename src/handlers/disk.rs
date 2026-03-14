use actix_web::{get, web, HttpResponse, Responder};
use crate::models::DiskInfo;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/disk",
    responses((status = 200, description = "Disk usage info", body = Vec<DiskInfo>))
)]
#[get("/disk")]
pub async fn disk_info(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let mounts: String = runner.execCommand(
        "cat /proc/mounts", false
    ).await.map(|r| r.stdout).unwrap_or_default();

    let mut disks: Vec<DiskInfo> = Vec::new();
    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let device: String = parts[0].to_string();
        let mount_point: String = parts[1].to_string();
        let fs_type: String = parts.get(2).unwrap_or(&"unknown").to_string();

        let stat_output: String = runner.execCommand(
            &format!(r#"stat -f -c '%S %b %f %a %i %I' {}"#, mount_point), false,
        ).await.map(|r| r.stdout).unwrap_or_default();

        let mut block_size: u64 = 1u64;
        let mut total_blocks: u64 = 0u64;
        let mut free_blocks: u64 = 0u64;
        let mut inode_total: Option<u64> = None;
        let mut inode_free: Option<u64> = None;

        let stat_parts: Vec<&str> = stat_output.split_whitespace().collect();
        if stat_parts.len() >= 6 {
            block_size = stat_parts[0].parse().unwrap_or(1);
            total_blocks = stat_parts[1].parse().unwrap_or(0);
            let free_blocks_tmp = stat_parts[3].parse().unwrap_or(0);
            free_blocks = free_blocks_tmp;
            let inode_total_tmp = stat_parts[4].parse().ok();
            let inode_free_tmp = stat_parts[5].parse().ok();
            inode_total = inode_total_tmp;
            inode_free = inode_free_tmp;
        }

        let used_blocks: u64 = total_blocks.saturating_sub(free_blocks);
        let size: String = format!("{}K", total_blocks * block_size / 1024);
        let used: String = format!("{}K", used_blocks * block_size / 1024);
        let avail: String = format!("{}K", free_blocks * block_size / 1024);
        let used_percentage = if total_blocks > 0 {
            format!("{}%", used_blocks * 100 / total_blocks)
        } else {
            "0%".to_string()
        };

        let inode_used: Option<u64> = inode_total
            .zip(inode_free)
            .map(|(t, f)| t.saturating_sub(f));

        let uuid: Option<String> = runner.execCommand(
            &format!("blkid -s UUID -o value {}", device), false
        ).await.map(|r| r.stdout.trim().to_string()).ok().filter(|s| !s.is_empty());

        disks.push(DiskInfo {
            filesystem: device.clone(),
            mount_point,
            size,
            used,
            avail,
            used_percentage,
            fs_type,
            inode_total,
            inode_used,
            inode_free,
            uuid,
            device: Some(device),
        });
    }

    HttpResponse::Ok().json(disks)
}