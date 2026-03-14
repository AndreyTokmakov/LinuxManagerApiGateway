use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::models::JournalEntry;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[derive(Deserialize)]
pub struct JournalQuery
{
    pub priority: Option<u8>,
    pub since: Option<String>,
    pub lines: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/journal",
    params(
        ("priority" = Option<u8>, Query, description = "max log priority (0-7)"),
        ("since" = Option<String>, Query, description = "since time"),
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
    let cmd: String = build_journal_command(None, &query);
    let output: String = run_journal(&runner, cmd).await;
    HttpResponse::Ok().json(parse_journal(output))
}

#[utoipa::path(
    get,
    path = "/journal/errors",
    responses(
        (status = 200, description = "Journal errors", body = Vec<JournalEntry>)
    )
)]
#[get("/journal/errors")]
pub async fn journal_errors(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let cmd: String = String::from("journalctl --no-pager --output=short -p 3 -n 200");
    let output: String = run_journal(&runner, cmd.to_string()).await;
    HttpResponse::Ok().json(parse_journal(output))
}

#[utoipa::path(
    get,
    path = "/journal/service/{name}",
    params(
        ("name" = String, Path, description = "systemd service name")
    ),
    responses(
        (status = 200, description = "Service journal logs", body = Vec<JournalEntry>)
    )
)]
#[get("/journal/service/{name}")]
pub async fn journal_service(runner: web::Data<SshCommandRunner>,
                             path: web::Path<String>,
                             query: web::Query<JournalQuery>) -> impl Responder
{
    let service: String = path.into_inner();
    let cmd: String = build_journal_command(Some(service), &query);
    let output: String = run_journal(&runner, cmd).await;
    HttpResponse::Ok().json(parse_journal(output))
}

async fn run_journal(runner: &SshCommandRunner,
                     cmd: String) -> String
{
    runner.execCommand(&cmd, true)
        .await.map(|r| r.stdout).unwrap_or_default()
}

fn build_journal_command(service: Option<String>,
                         query: &JournalQuery) -> String
{
    let mut cmd: String = String::from("journalctl --no-pager --output=short");

    if let Some(service) = service {
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
    cmd
}

fn parse_journal(raw: String) -> Vec<JournalEntry>
{
    let mut entries = Vec::new();
    for line in raw.lines()
    {
        let parts: Vec<&str> = line.splitn(5, ' ').collect();
        if parts.len() < 5 {
            continue;
        }

        let timestamp: String = format!("{} {} {}", parts[0], parts[1], parts[2]);
        let hostname: String = parts[3].to_string();

        let rest: &str = parts[4];
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