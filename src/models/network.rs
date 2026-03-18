use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct NetworkInterface
{
    pub name: String,
    pub mac_address: String,
    pub state: String,
    pub mtu: u32,
    pub ip_addresses: Vec<String>,
    pub rx_bytes: Option<u64>,
    pub tx_bytes: Option<u64>,
    pub speed: Option<String>,
    pub driver: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct OpenPort
{
    pub protocol: String,
    pub local_address: String,
    pub port: u16,
    pub state: String,
}

#[derive(Serialize, ToSchema)]
pub struct NetworkRoute
{
    pub interface: String,
    pub destination: String,
    pub gateway: String,
    pub mask: String,
}