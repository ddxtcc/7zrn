use std::collections::VecDeque;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::archive::Listing;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobKind {
    List,
    Extract,
    Add,
    Delete,
    Test,
    Create,
    Modify,
    Checksum,
    Verify,
}

#[derive(Clone, Debug)]
pub enum JobPayload {
    Listed(Listing),
    Extracted {
        dest: std::path::PathBuf,
        count: usize,
        names: Vec<String>,
    },
    Opened {
        path: std::path::PathBuf,
    },
    Modified,
    Tested {
        summary: String,
    },
    Created {
        archive: std::path::PathBuf,
    },
    Nested {
        path: std::path::PathBuf,
        title: String,
    },
    Checksums(Vec<crate::checksum::ChecksumEntry>),
    MovedOut {
        dest: std::path::PathBuf,
        count: usize,
        names: Vec<String>,
    },
    Verified {
        summary: String,
        failed: usize,
        missing: usize,
    },
}

pub const CANCELLED: &str = "Operation cancelled";

pub struct Job {
    pub kind: JobKind,
    pub label: String,
    pub percent: Option<u8>,
    pub log: VecDeque<String>,
    pub done: bool,
    pub cancel: bool,
    pub pid: Option<u32>,
    pub result: Option<Result<JobPayload, String>>,
}

pub type SharedJob = Arc<Mutex<Job>>;

pub fn new_job(kind: JobKind, label: impl Into<String>) -> SharedJob {
    Arc::new(Mutex::new(Job {
        kind,
        label: label.into(),
        percent: None,
        log: VecDeque::new(),
        done: false,
        cancel: false,
        pid: None,
        result: None,
    }))
}

pub fn note(job: &SharedJob, line: impl Into<String>) {
    let mut job = job.lock().unwrap();
    push_log(&mut job, line.into());
}

pub fn finish(job: &SharedJob, result: Result<JobPayload, String>) {
    let mut job = job.lock().unwrap();
    job.done = true;
    job.pid = None;
    job.result = Some(result);
}

pub fn cancel(job: &SharedJob) {
    let mut job = job.lock().unwrap();
    if job.done || job.cancel {
        return;
    }
    job.cancel = true;
    push_log(&mut job, "Cancelling operation...".to_string());
}

fn push_log(job: &mut Job, line: String) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }
    if job.log.len() >= 400 {
        job.log.pop_front();
    }
    job.log.push_back(line.to_string());
}

fn parse_percent(line: &str) -> Option<u8> {
    let trimmed = line.trim_start();
    let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    if !trimmed[digits.len()..].starts_with('%') {
        return None;
    }
    digits.parse::<u8>().ok().filter(|p| *p <= 100)
}

pub fn run_command(cmd: &mut Command, job: &SharedJob) -> Result<(), String> {
    run_impl(cmd, job, false).map(|_| ())
}

pub fn run_capture(cmd: &mut Command, job: &SharedJob) -> Result<String, String> {
    run_impl(cmd, job, true)
}

fn run_impl(cmd: &mut Command, job: &SharedJob, capture_stdout: bool) -> Result<String, String> {
    if job.lock().unwrap().cancel {
        return Err(CANCELLED.to_string());
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.env("LC_ALL", "C.UTF-8").env("LANG", "C.UTF-8");
    let program = cmd.get_program().to_string_lossy().into_owned();
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start {program}: {e}"))?;
    job.lock().unwrap().pid = Some(child.id());
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");

    let out_shared = Arc::new(Mutex::new(Vec::<u8>::new()));
    let err_shared = Arc::new(Mutex::new(Vec::<u8>::new()));

    let out_job = job.clone();
    let out_buf = out_shared.clone();
    let out_thread = thread::spawn(move || {
        read_stream(stdout, out_buf, out_job, capture_stdout, true);
    });

    let err_job = job.clone();
    let err_buf = err_shared.clone();
    let err_thread = thread::spawn(move || {
        read_stream(stderr, err_buf, err_job, true, false);
    });

    let status = loop {
        if job.lock().unwrap().cancel {
            let _ = child.kill();
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(error) => return Err(format!("Failed while running {program}: {error}")),
        }
    };
    let _ = out_thread.join();
    let _ = err_thread.join();
    let cancelled = job.lock().unwrap().cancel;
    job.lock().unwrap().pid = None;
    if cancelled {
        return Err(CANCELLED.to_string());
    }

    let stdout = String::from_utf8_lossy(&out_shared.lock().unwrap()).to_string();
    let stderr = String::from_utf8_lossy(&err_shared.lock().unwrap()).to_string();

    if !status.success() {
        let code = status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "signal".to_string());
        let tail = {
            let job = job.lock().unwrap();
            let mut lines: Vec<&str> = job.log.iter().rev().take(12).map(|s| s.as_str()).collect();
            lines.reverse();
            lines.join("\n")
        };
        let detail = if stderr.trim().is_empty() {
            tail
        } else {
            format!("{stderr}\n{tail}")
        };
        return Err(format!(
            "Command failed (exit {code}): {program}\n{}",
            trim_for_display(&detail)
        ));
    }

    Ok(stdout)
}

fn trim_for_display(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() > 20 {
        lines = lines.split_off(lines.len() - 20);
    }
    lines.join("\n")
}

fn read_stream<R: Read>(
    mut reader: R,
    raw: Arc<Mutex<Vec<u8>>>,
    job: SharedJob,
    capture_output: bool,
    is_stdout: bool,
) {
    let mut chunk = [0u8; 16 * 1024];
    let mut pending: Vec<u8> = Vec::new();
    loop {
        let n = match reader.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        raw.lock().unwrap().extend_from_slice(&chunk[..n]);
        for &byte in &chunk[..n] {
            if byte == b'\n' || byte == b'\r' {
                if !pending.is_empty() {
                    handle_line(&pending, &job, capture_output, is_stdout);
                    pending.clear();
                }
            } else {
                pending.push(byte);
                if pending.len() > 128 * 1024 {
                    handle_line(&pending, &job, capture_output, is_stdout);
                    pending.clear();
                }
            }
        }
    }
    if !pending.is_empty() {
        handle_line(&pending, &job, capture_output, is_stdout);
    }
}

fn handle_line(bytes: &[u8], job: &SharedJob, capture_output: bool, is_stdout: bool) {
    let line = String::from_utf8_lossy(bytes);
    let line = line.trim();
    if line.is_empty() {
        return;
    }
    if is_stdout && capture_output {
        return;
    }
    if is_stdout && let Some(pct) = parse_percent(line) {
        let mut job = job.lock().unwrap();
        job.percent = Some(pct);
        return;
    }
    let mut job = job.lock().unwrap();
    push_log(&mut job, line.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancels_a_running_command() {
        let job = new_job(JobKind::Extract, "cancel test");
        let worker_job = job.clone();
        let worker = thread::spawn(move || {
            let mut command = Command::new("sh");
            command.args(["-c", "sleep 5"]);
            run_command(&mut command, &worker_job)
        });
        thread::sleep(Duration::from_millis(150));
        cancel(&job);
        let result = worker.join().unwrap();
        assert_eq!(result.unwrap_err(), CANCELLED);
    }

    #[test]
    fn parses_7z_progress_lines() {
        assert_eq!(parse_percent("  0% 1 - file"), Some(0));
        assert_eq!(parse_percent(" 47% 12 - foo"), Some(47));
        assert_eq!(parse_percent("100% 20"), Some(100));
        assert_eq!(parse_percent("Folders: 2"), None);
        assert_eq!(parse_percent("12x"), None);
    }
}
