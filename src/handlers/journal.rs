use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::models::JournalEntry;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[derive(Deserialize)]
pub struct JournalQuery
{
    pub service: Option<String>,
    pub priority: Option<u8>,
    pub since: Option<String>,
    pub lines: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/journal",
    params(
        ("service" = Option<String>, Query, description = "systemd service name"),
        ("priority" = Option<u8>, Query, description = "max log priority (0-7)"),
        ("since" = Option<String>, Query, description = "since time (e.g. 1h, today, yesterday)"),
        ("lines" = Option<u32>, Query, description = "number of log lines")
    ),
    responses(
        (status = 200, description = "Journal logs", body = Vec<JournalEntry>)
    )
)]
#[get("/journal")]
pub async fn journal_logs(runner: web::Data<SshCommandRunner>,
                          query: web::Query<JournalQuery>) -> impl Responder
{
    let mut cmd: String = String::from("journalctl --no-pager --output=short");

    if let Some(service) = &query.service {
        cmd.push_str(&format!(" -u {}", service));
    }
    if let Some(priority) = query.priority {
        cmd.push_str(&format!(" -p {}", priority));
    }
    if let Some(since) = &query.since {
        cmd.push_str(&format!(" --since '{}'", since));
    }
    if let Some(lines) = query.lines {
        cmd.push_str(&format!(" -n {}", lines));
    } else {
        cmd.push_str(" -n 100");
    }

    let output: String = runner.execCommand(&cmd, true).await
        .map(|r| r.stdout).unwrap_or_default();
    let logs: Vec<JournalEntry>  = parse_journal(output);
    HttpResponse::Ok().json(logs)
}

fn parse_journal(raw: String) -> Vec<JournalEntry>
{
    let mut entries: Vec<JournalEntry> = Vec::new();
    for line in raw.lines()
    {
        let parts: Vec<&str> = line.splitn(5, ' ').collect();
        if parts.len() < 5 {
            continue;
        }

        let timestamp = format!("{} {} {}", parts[0], parts[1], parts[2]);
        let hostname = parts[3].to_string();

        let rest = parts[4];

        let (unit, message) = if let Some(idx) = rest.find(':') {
            let (u, m) = rest.split_at(idx);
            (Some(u.trim().to_string()), m[1..].trim().to_string())
        } else {
            (None, rest.to_string())
        };

        entries.push(JournalEntry
        {
            timestamp,
            hostname,
            unit,
            priority: None,
            message
        });
    }

    entries
}