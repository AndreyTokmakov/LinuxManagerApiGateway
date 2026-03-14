use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct JournalEntry
{
    pub timestamp: String,
    pub hostname: String,
    pub unit: Option<String>,
    pub priority: Option<String>,
    pub message: String
}