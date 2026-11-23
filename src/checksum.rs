use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crc32fast::Hasher as Crc32;
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub(crate) struct ChecksumEntry {
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) sha256: String,
    pub(crate) sha1: String,
    pub(crate) md5: String,
    pub(crate) crc32: String,
}

pub(crate) fn hash_files(paths: &[PathBuf]) -> Result<Vec<ChecksumEntry>, String> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            collect_files(path, &mut files)?;
        } else if path.is_file() {
            files.push(path.clone());
        }
    }
    files.sort();
    files.dedup();
    let mut entries = Vec::new();
    for path in files {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file")
            .to_string();
        entries.push(hash_file_with_name(&path, name)?);
    }
    Ok(entries)
}

pub(crate) fn hash_tree(root: &Path) -> Result<Vec<ChecksumEntry>, String> {
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    files.sort();
    let mut entries = Vec::new();
    for path in files {
        let name = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        entries.push(hash_file_with_name(&path, name)?);
    }
    Ok(entries)
}

#[derive(Clone, Debug)]
pub(crate) struct Verification {
    pub(crate) passed: usize,
    pub(crate) failed: usize,
    pub(crate) missing: usize,
    pub(crate) details: Vec<String>,
}

impl Verification {
    pub(crate) fn summary(&self) -> String {
        let mut text = format!(
            "Checked {} file(s): {} passed, {} failed, {} missing.",
            self.passed + self.failed + self.missing,
            self.passed,
            self.failed,
            self.missing
        );
        if !self.details.is_empty() {
            text.push_str("\n\n");
            text.push_str(&self.details.join("\n"));
        }
        text
    }
}

pub(crate) fn looks_like_manifest(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    name == "sha256sums"
        || name.starts_with("checksums")
        || name.ends_with(".sha256")
        || name.ends_with(".sha1")
        || name.ends_with(".md5")
        || name.ends_with(".sha256sum")
}

pub(crate) fn verify_manifest(path: &Path) -> Result<Verification, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let parent = path.parent().unwrap_or(Path::new("."));
    let mut verification = Verification {
        passed: 0,
        failed: 0,
        missing: 0,
        details: Vec::new(),
    };
    let mut checked = 0;
    for (line_number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((hash, name)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let name = name.trim().trim_start_matches('*');
        if name.is_empty() {
            continue;
        }
        checked += 1;
        let target = parent.join(name);
        if !target.is_file() {
            verification.missing += 1;
            verification.details.push(format!("missing  {name}"));
            continue;
        }
        let entry = hash_file_with_name(&target, name.to_string())?;
        let actual = match hash.len() {
            64 => &entry.sha256,
            40 => &entry.sha1,
            32 => &entry.md5,
            8 => &entry.crc32,
            _ => {
                verification.details.push(format!(
                    "line {}: unsupported hash length {}",
                    line_number + 1,
                    hash.len()
                ));
                verification.failed += 1;
                continue;
            }
        };
        if actual.eq_ignore_ascii_case(hash) {
            verification.passed += 1;
        } else {
            verification.failed += 1;
            verification.details.push(format!("FAILED  {name}"));
        }
    }
    if checked == 0 {
        return Err("No checksum entries found.".into());
    }
    Ok(verification)
}

pub(crate) fn manifest(entries: &[ChecksumEntry]) -> String {
    let mut text = String::new();
    for entry in entries {
        text.push_str(&format!("{}  {}\n", entry.sha256, entry.name));
    }
    text
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("Could not read {}: {error}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("Could not read directory: {error}"))?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            collect_files(&path, files)?;
        } else if meta.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn hash_file_with_name(path: &Path, name: String) -> Result<ChecksumEntry, String> {
    let mut file =
        File::open(path).map_err(|error| format!("Could not open {}: {error}", path.display()))?;
    let mut sha256 = Sha256::new();
    let mut sha1 = Sha1::new();
    let mut md5 = Md5::new();
    let mut crc = Crc32::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut size = 0u64;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        size += read as u64;
        let chunk = &buffer[..read];
        sha256.update(chunk);
        sha1.update(chunk);
        md5.update(chunk);
        crc.update(chunk);
    }
    Ok(ChecksumEntry {
        name,
        size,
        sha256: format!("{:x}", sha256.finalize()),
        sha1: format!("{:x}", sha1.finalize()),
        md5: format!("{:x}", md5.finalize()),
        crc32: format!("{:08x}", crc.finalize()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn verifies_a_manifest() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("7zrn-verify-{}-{stamp}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let file = directory.join("a.txt");
        fs::write(&file, b"abc").unwrap();
        let manifest = directory.join("SHA256SUMS");
        fs::write(
            &manifest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  a.txt\n",
        )
        .unwrap();
        let verification = verify_manifest(&manifest).unwrap();
        assert_eq!(verification.passed, 1);
        assert_eq!(verification.failed, 0);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn hashes_known_content() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("7zrn-hash-{}-{stamp}", std::process::id()));
        fs::write(&path, b"abc").unwrap();
        let entries = hash_files(std::slice::from_ref(&path)).unwrap();
        assert_eq!(
            entries[0].sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(entries[0].sha1, "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(entries[0].md5, "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(entries[0].crc32, "352441c2");
        let _ = fs::remove_file(path);
    }
}
