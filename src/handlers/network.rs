use actix_web::{get, web, HttpResponse, Responder};
use crate::models::*;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/network/interfaces",
    responses((status = 200, description = "Network interfaces", body = Vec<NetworkInterface>))
)]
#[get("/network/interfaces")]
pub async fn interfaces(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let cmd: &str = r#"
for iface in /sys/class/net/*; do
    name=$(basename $iface)
    mac=$(cat $iface/address 2>/dev/null)
    state=$(cat $iface/operstate 2>/dev/null)
    mtu=$(cat $iface/mtu 2>/dev/null)
    rx=$(cat $iface/statistics/rx_bytes 2>/dev/null)
    tx=$(cat $iface/statistics/tx_bytes 2>/dev/null)
    speed=$(cat $iface/speed 2>/dev/null)
    driver=$(basename $(readlink $iface/device/driver 2>/dev/null) 2>/dev/null)

    echo "$name|$mac|$state|$mtu|$rx|$tx|$speed|$driver"
done
"#;

    let output: String = runner.execCommand(cmd, false)
        .await.map(|r| r.stdout).unwrap_or_default();

    let mut interfaces: Vec<NetworkInterface> = Vec::new();
    for line in output.lines()
    {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 8 {
            continue;
        }

        interfaces.push(NetworkInterface {
            name: parts[0].to_string(),
            mac_address: parts[1].to_string(),
            state: parts[2].to_string(),
            mtu: parts[3].parse().unwrap_or(0),
            ip_addresses: Vec::new(),
            rx_bytes: parts[4].parse().ok(),
            tx_bytes: parts[5].parse().ok(),
            speed: if parts[6].is_empty() { None } else { Some(parts[6].to_string()) },
            driver: if parts[7].is_empty() { None } else { Some(parts[7].to_string()) },
        });
    }

    HttpResponse::Ok().json(interfaces)
}


#[utoipa::path(
    get,
    path = "/network/ports",
    responses((status = 200, description = "Open ports", body = Vec<OpenPort>))
)]
#[get("/network/ports")]
pub async fn open_ports(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let output: String = runner.execCommand("cat /proc/net/tcp /proc/net/tcp6", false)
        .await.map(|r| r.stdout).unwrap_or_default();

    let mut ports: Vec<OpenPort> = Vec::new();
    for line in output.lines()
    {
        let line: &str = line.trim_start();
        if line.starts_with("sl") || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let local: &str = parts[1];
        let state: &str = parts[3];

        let addr_port: Vec<&str> = local.split(':').collect();
        if addr_port.len() != 2 {
            continue;
        }

        let ip_hex: &str = addr_port[0];
        let port_hex: &str = addr_port[1];

        let port: u16 = u16::from_str_radix(port_hex, 16).unwrap_or(0);
        let ip: String = if ip_hex.len() == 8 {
            decode_ipv4(ip_hex).unwrap_or_else(|| ip_hex.to_string())
        } else {
            ip_hex.to_string()
        };

        let state_str: &str = decode_tcp_state(state);

        ports.push(OpenPort {
            protocol: "tcp".to_string(),
            local_address: ip,
            port,
            state: state_str.to_string(),
        });
    }

    HttpResponse::Ok().json(ports)
}


fn decode_ipv4(hex: &str) -> Option<String>
{
    if hex.len() != 8 {
        return None;
    }

    let ip: u32 = u32::from_str_radix(hex, 16).ok()?;
    let b1: u8 = (ip & 0xff) as u8;
    let b2: u8 = ((ip >> 8) & 0xff) as u8;
    let b3: u8 = ((ip >> 16) & 0xff) as u8;
    let b4: u8 = ((ip >> 24) & 0xff) as u8;

    Some(format!("{}.{}.{}.{}", b1, b2, b3, b4))
}

fn decode_tcp_state(state: &str) -> &'static str
{
    match state {
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0A" => "LISTEN",
        "0B" => "CLOSING",
        _ => "OTHER",
    }
}