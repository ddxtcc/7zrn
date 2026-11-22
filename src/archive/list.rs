use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use super::commands::apply_password;
use super::{Entry, Format, Listing, SEVEN_ZIP, display_archive_path};

pub fn list_archive(path: &Path, password: Option<&str>) -> Result<Listing, String> {
    let format = Format::detect(path);
    let entries = if format.is_compressed_tar() {
        list_compressed_tar(path, format)?
    } else {
        list_with_7z(path, password)?
    };
    let archive_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    Ok(Listing {
        entries,
        archive_size,
    })
}

fn list_with_7z(path: &Path, password: Option<&str>) -> Result<Vec<Entry>, String> {
    let mut cmd = Command::new(SEVEN_ZIP);
    cmd.args(["l", "-slt", "-ba", "-sccUTF-8"]);
    apply_password(&mut cmd, password);
    cmd.arg("--")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("LC_ALL", "C.UTF-8")
        .env("LANG", "C.UTF-8");
    let output = cmd
        .output()
        .map_err(|e| format!("Could not run {SEVEN_ZIP}: {e}"))?;
    if !output.status.success() {
        return Err(archive_open_error(
            path,
            &String::from_utf8_lossy(&output.stderr),
            password.is_some(),
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_slt(&text)
}

fn list_compressed_tar(path: &Path, format: Format) -> Result<Vec<Entry>, String> {
    let decompressor = format
        .decompressor()
        .ok_or_else(|| "Unknown compression for tar archive".to_string())?;
    let mut decomp = Command::new(decompressor)
        .arg("-dc")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not run {decompressor}: {e}"))?;
    let decomp_stdout = decomp
        .stdout
        .take()
        .ok_or_else(|| "Could not read decompressor output".to_string())?;

    let seven = Command::new(SEVEN_ZIP)
        .args(["l", "-slt", "-ba", "-sccUTF-8", "-si", "-ttar"])
        .stdin(Stdio::from(decomp_stdout))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("LC_ALL", "C.UTF-8")
        .env("LANG", "C.UTF-8")
        .spawn()
        .map_err(|e| format!("Could not run {SEVEN_ZIP}: {e}"))?;

    let seven_output = seven
        .wait_with_output()
        .map_err(|e| format!("Failed while reading archive: {e}"))?;
    let decomp_output = decomp
        .wait_with_output()
        .map_err(|e| format!("Failed while decompressing archive: {e}"))?;

    if !decomp_output.status.success() {
        return Err(format!(
            "Could not decompress {}:\n{}",
            path.display(),
            String::from_utf8_lossy(&decomp_output.stderr).trim()
        ));
    }
    if !seven_output.status.success() {
        return Err(archive_open_error(
            path,
            &String::from_utf8_lossy(&seven_output.stderr),
            false,
        ));
    }
    parse_slt(&String::from_utf8_lossy(&seven_output.stdout))
}

fn archive_open_error(path: &Path, stderr: &str, password_was_provided: bool) -> String {
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("is not archive")
        || lower.contains("cannot open the file as")
        || lower.contains("cannot open as archive")
    {
        format!("{} is not a supported archive.", path.display())
    } else if lower.contains("password") || lower.contains("encrypted") {
        if password_was_provided {
            format!("Wrong password for {}.", path.display())
        } else {
            format!("Password required for {}.", path.display())
        }
    } else {
        let detail = stderr
            .lines()
            .filter(|l| !l.trim().is_empty())
            .take(8)
            .collect::<Vec<_>>()
            .join("\n");
        if detail.is_empty() {
            format!("Could not read {}.", path.display())
        } else {
            format!("Could not read {}.\n{detail}", path.display())
        }
    }
}
pub fn parse_slt(text: &str) -> Result<Vec<Entry>, String> {
    let mut entries = Vec::new();
    let mut fields: HashMap<String, String> = HashMap::new();

    fn flush(fields: &mut HashMap<String, String>, entries: &mut Vec<Entry>) {
        if fields.is_empty() {
            return;
        }
        if let Some(entry) = entry_from_fields(fields) {
            entries.push(entry);
        }
        fields.clear();
    }

    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            flush(&mut fields, &mut entries);
            continue;
        }
        if line.starts_with("----------") {
            continue;
        }
        if let Some((key, value)) = line.split_once(" = ") {
            fields.insert(key.trim().to_string(), value.to_string());
        } else if let Some((key, value)) = line.split_once('=') {
            fields.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    flush(&mut fields, &mut entries);
    Ok(entries)
}

fn entry_from_fields(fields: &mut HashMap<String, String>) -> Option<Entry> {
    let raw_path = fields.get("Path")?.clone();
    if raw_path.trim().is_empty() {
        return None;
    }
    let display_path = display_archive_path(&raw_path);
    if display_path.is_empty() {
        return None;
    }
    let attributes = fields
        .get("Attributes")
        .map(|s| s.trim_start())
        .unwrap_or("");
    let folder = fields
        .get("Folder")
        .map(|f| f.trim() == "+")
        .unwrap_or(false);
    let is_dir = folder || attributes.starts_with('D') || raw_path.ends_with('/');
    let symlink_target = fields
        .get("Symbolic Link")
        .filter(|s| !s.trim().is_empty())
        .cloned();
    let size = fields
        .get("Size")
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let packed = fields.get("Packed Size").and_then(|s| {
        let s = s.trim();
        if s.is_empty() {
            None
        } else {
            s.parse::<u64>().ok()
        }
    });
    let modified = fields
        .get("Modified")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.split('.').next().unwrap_or(s).to_string());
    let method = fields
        .get("Method")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let encrypted = fields
        .get("Encrypted")
        .map(|s| s.trim() == "+")
        .unwrap_or(false);

    Some(Entry {
        raw_path,
        display_path,
        is_dir,
        is_symlink: symlink_target.is_some(),
        symlink_target,
        size,
        packed,
        modified,
        method,
        encrypted,
    })
}
