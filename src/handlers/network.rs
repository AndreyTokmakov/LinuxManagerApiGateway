use actix_web::{get, web, HttpResponse, Responder};
use clap::builder::Str;
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/network/interfaces",
    responses((status = 200, description = "Network interfaces", body = Vec<NetworkInterface>))
)]
#[get("/network/interfaces")]
pub async fn interfaces(runner: web::Data<SshCommandRunner>) ->impl Responder
{
    let output = runner.execCommand(
        "ip -o link show", false
    ).await.map(|r| r.stdout).unwrap_or_default();

    let mut interfaces: Vec<NetworkInterface> = Vec::new();
    for line in output.lines() {
        // Простейший разбор: 2: eth0: <BROADCAST,...> mtu 1500 ...
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 17 {
            let name: String = parts[1].trim_end_matches(':').to_string();
            let state: String = parts[8].to_string();
            let mtu: u32 = parts[4].parse().unwrap_or(0);
            let mac_address: String = parts[15].to_string();

            // Get IP address
            let ip_out: String = runner.execCommand(
                &format!("ip -o addr show {}", name), false
            ).await.map(|r| r.stdout).unwrap_or_default();

            let ip_addresses: Vec<String> = ip_out.lines()
                .filter_map(|l| l.split_whitespace().nth(3))
                .map(|s| s.to_string())
                .collect();

            // RX/TX
            let rx_bytes: Option<u64> = runner.execCommand(
                &format!("cat /sys/class/net/{}/statistics/rx_bytes", name), false
            ).await.ok().and_then(|r| r.stdout.trim().parse().ok());

            let tx_bytes: Option<u64> = runner.execCommand(
                &format!("cat /sys/class/net/{}/statistics/tx_bytes", name), false
            ).await.ok().and_then(|r| r.stdout.trim().parse().ok());

            // Скорость и драйвер
            let speed: Option<String> = runner.execCommand(
                &format!("ethtool {} | grep Speed", name), false
            ).await.ok().map(|r| r.stdout.trim().to_string());

            let driver: Option<String> = runner.execCommand(
                &format!("ethtool -i {}", name), false
            ).await.ok().and_then(|r| {
                for l in r.stdout.lines() {
                    if let Some(v) = l.strip_prefix("driver: ") {
                        return Some(v.to_string());
                    }
                }
                None
            });

            interfaces.push(NetworkInterface {
                name,
                mac_address,
                state,
                mtu,
                ip_addresses,
                rx_bytes,
                tx_bytes,
                speed,
                driver,
            });
        }
    }

    HttpResponse::Ok().json(interfaces)
}