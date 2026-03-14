use actix_web::{get, web, HttpResponse, Responder};
use tokio::time::{sleep, Duration};
use std::collections::HashMap;

use crate::models::ProcessInfo;
use crate::ssh_connection_pool::ssh_connection_pool::SshCommandRunner;

async fn execCommandAsync(runner: &SshCommandRunner, cmd: &str, sudo: bool) -> String
{
    runner.execCommand(cmd, sudo)
        .await
        .map(|r| r.stdout.trim().to_string())
        .unwrap_or_default()
}

fn parse_total_cpu(stat: &str) -> u64
{
    stat.lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .skip(1)
        .filter_map(|v| v.parse::<u64>().ok())
        .sum()
}

fn parse_mem_total(mem_info: &str) -> u64
{
    for line in mem_info.lines() {
        if line.starts_with("MemTotal:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                return parts[1].parse::<u64>().unwrap_or(0) * 1024;
            }
        }
    }
    return 0
}

async fn get_pid_list(runner: &SshCommandRunner) -> Vec<u32>
{
    let output: String = execCommandAsync(&runner, "ls /proc", false).await;
    output.lines().filter_map(|line| line.parse::<u32>().ok()).collect()
}

async fn read_process_cpu(runner: &SshCommandRunner, pid: u32) -> u64
{
    let output: String = execCommandAsync(&runner, &format!("cat /proc/{}/stat", pid), false).await;
    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() > 15 {
        let u_time: u64 = parts[13].parse::<u64>().unwrap_or(0);
        let s_time: u64 = parts[14].parse::<u64>().unwrap_or(0);
        u_time + s_time
    } else {
        0
    }
}

async fn read_process_info(runner: &SshCommandRunner,
                           pid: u32,
                           total_mem_bytes: u64,
                           cpu1: u64,
                           delta_total: u64) -> Option<ProcessInfo>
{
    let output: String = execCommandAsync(&runner, &format!("cat /proc/{}/stat", pid), false).await;
    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() < 22 {
        return None;
    }

    let ppid: u32 = parts[3].parse::<u32>().unwrap_or(0);
    let state: String = parts[2].to_string();
    let utime: u64 = parts[13].parse::<u64>().unwrap_or(0);
    let stime: u64 = parts[14].parse::<u64>().unwrap_or(0);
    let start_time: u64 = parts[21].parse::<u64>().unwrap_or(0);

    let delta_proc: u64 = (utime + stime).saturating_sub(cpu1);

    let cpu: f64 = if delta_total > 0 {
        (delta_proc as f64 / delta_total as f64) * 100.0
    } else {
        0.0
    };

    // memory
    let statm: String = execCommandAsync(&runner, &format!("cat /proc/{}/statm", pid), false).await;
    let statm_parts: Vec<&str> = statm.split_whitespace().collect();
    let rss_pages: u64 = statm_parts.get(1).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    let vsize_pages: u64 = statm_parts.get(0).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);

    let rss_kb: u64 = rss_pages * 4;
    let vsize_kb: u64 = vsize_pages * 4;

    let mem: f64 = if total_mem_bytes > 0 {
        ((rss_kb * 1024) as f64 / total_mem_bytes as f64) * 100.0
    } else { 0.0 };

    let status: String = execCommandAsync(&runner,
        &format!("cat /proc/{}/status", pid), false
    ).await;

    let mut threads: u32 = 0;
    for line in status.lines() {
        if line.starts_with("Threads:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                threads = parts[1].parse::<u32>().unwrap_or(0);
            }
        }
    }

    let cmdline_raw: String = execCommandAsync(&runner, &format!("cat /proc/{}/cmdline", pid), false).await;
    let mut cmd: String = cmdline_raw.replace('\0', " ").trim().to_string();
    if cmd.is_empty() {
        cmd = execCommandAsync(&runner, &format!("cat /proc/{}/comm", pid), false).await;
    }

    Some(ProcessInfo {
        pid,
        ppid,
        state,
        cpu,
        mem,
        rss_kb,
        vsize_kb,
        threads,
        start_time,
        cmd,
    })
}

#[utoipa::path(
    get,
    path = "/process",
    responses((status = 200, description = "Running processes", body = Vec<ProcessInfo>))
)]
#[get("/process")]
pub async fn process_list(runner: web::Data<SshCommandRunner>) -> impl Responder
{
    let pids: Vec<u32> = get_pid_list(&runner).await;
    let mem_info: String = execCommandAsync(&runner, "cat /proc/meminfo", false).await;
    let total_mem: u64 = parse_mem_total(&mem_info);

    // first CPU snapshot
    let cpu_stat1: String = execCommandAsync(&runner, "cat /proc/stat", false).await;
    let total_cpu1: u64 = parse_total_cpu(&cpu_stat1);
    let mut proc_cpu1: HashMap<u32, u64> = HashMap::new();
    for &pid in &pids {
        proc_cpu1.insert(pid, read_process_cpu(&runner, pid).await);
    }

    sleep(Duration::from_millis(200)).await;

    // second CPU snapshot
    let cpu_stat2: String = execCommandAsync(&runner, "cat /proc/stat", false).await;
    let total_cpu2: u64 = parse_total_cpu(&cpu_stat2);
    let delta_total: u64 = total_cpu2.saturating_sub(total_cpu1);

    // collect process info
    let mut processes = Vec::new();
    for &pid in &pids {
        let cpu1: u64 = *proc_cpu1.get(&pid).unwrap_or(&0);
        if let Some(info) = read_process_info(&runner, pid, total_mem, cpu1, delta_total).await {
            processes.push(info);
        }
    }

    HttpResponse::Ok().json(processes)
}