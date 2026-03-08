use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct NetworkInterface {
    pub name: String,
    pub address: String,
}