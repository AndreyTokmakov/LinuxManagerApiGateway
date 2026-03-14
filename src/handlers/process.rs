use actix_web::{get, web, HttpResponse, Responder};
use clap::builder::Str;
use crate::models::ProcessInfo;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

#[utoipa::path(
    get,
    path = "/process",
    responses((status = 200, description = "List of processes", body = Vec<ProcessInfo>))
)]
#[get("/process")]
pub async fn process_list(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let cpu_total: u64 = read_total_cpu(&runner).await;
    let raw: String = read_all_processes(&runner).await;
    let processes: Vec<ProcessInfo> = parse_processes(raw, cpu_total);
    HttpResponse::Ok().json(processes)
}

async fn read_total_cpu(runner: &SshCommandRunner) -> u64
{
    let output: String = runner.execCommand(
        "cat /proc/stat | grep '^cpu '", false
    ).await.map(|r| r.stdout).unwrap_or_default();

    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() < 8 {
        return 0;
    }

    parts[1..].iter()
        .filter_map(|v| v.parse::<u64>().ok())
        .sum()
}

async fn read_all_processes(runner: &SshCommandRunner) -> String
{
    let script = r#"
for pid in /proc/[0-9]*; do
  pid=${pid##*/}

  stat=$(cat /proc/$pid/stat 2>/dev/null) || continue
  statm=$(cat /proc/$pid/statm 2>/dev/null)
  threads=$(grep '^Threads:' /proc/$pid/status 2>/dev/null | awk '{print $2}')
  cmd=$(tr '\0' ' ' < /proc/$pid/cmdline 2>/dev/null)

  echo "$pid|$stat|$statm|$threads|$cmd"
done
"#;

    runner.execCommand(script, false)
        .await
        .map(|r| r.stdout)
        .unwrap_or_default()
}

fn parse_processes(raw: String, cpu_total: u64) -> Vec<ProcessInfo>
{
    let mut processes = Vec::new();
    for line in raw.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 5 {
            continue;
        }

        let pid: u32 = parts[0].parse().unwrap_or(0);
        let stat: &str = parts[1];
        let statm: &str = parts[2];
        let threads: u32 = parts[3].parse().unwrap_or(0);
        let cmd: String = parts[4].to_string().trim().to_string();
        let stat_parts: Vec<&str> = stat.split_whitespace().collect();
        if stat_parts.len() < 24 {
            continue;
        }

        let name: String = stat_parts[1]
            .trim_start_matches('(')
            .trim_end_matches(')')
            .to_string();

        let state: String = stat_parts[2].to_string();
        let ppid: u32 = stat_parts[3].parse().unwrap_or(0);
        let utime: u64 = stat_parts[13].parse().unwrap_or(0);
        let stime: u64 = stat_parts[14].parse().unwrap_or(0);
        let total_time: u64 = utime + stime;
        let cpu = if cpu_total > 0 {
            (total_time as f64 / cpu_total as f64) * 100.0
        } else {
            0.0
        };

        let statm_parts: Vec<&str> = statm.split_whitespace().collect();
        let page_size: u64 = 4096;
        let vsize_kb: u64 = statm_parts.get(0)
            .and_then(|v| v.parse::<u64>().ok()).map(|v| v * page_size / 1024).unwrap_or(0);

        let rss_kb: u64 = statm_parts.get(1)
            .and_then(|v| v.parse::<u64>().ok()).map(|v| v * page_size / 1024).unwrap_or(0);

        let mem = rss_kb as f64 / 1024.0;

        let start_time: u64 = stat_parts
            .get(21)
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        processes.push(ProcessInfo
        {
            pid,
            ppid,
            state,
            name,
            cpu,
            mem,
            rss_kb,
            vsize_kb,
            threads,
            start_time,
            cmd
        });
    }

    processes
}