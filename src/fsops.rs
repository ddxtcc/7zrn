use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::archive;

#[derive(Clone, Debug)]
pub struct FsEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub symlink_target: Option<PathBuf>,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub is_parent: bool,
}

impl FsEntry {
    pub fn is_archive(&self) -> bool {
        !self.is_dir && archive::is_archive_name(&self.name)
    }

    pub fn type_label(&self) -> String {
        if self.is_dir {
            "folder".into()
        } else if self.is_symlink {
            "symlink".into()
        } else if self.is_archive() {
            archive::format_label(archive::Format::detect(&self.path)).into()
        } else {
            self.path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_uppercase())
                .unwrap_or_else(|| "file".into())
        }
    }
}

pub fn read_dir(path: &Path) -> Result<Vec<FsEntry>, String> {
    let iter = fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut entries = Vec::new();
    for item in iter {
        let Ok(item) = item else { continue };
        let name = item.file_name().to_string_lossy().into_owned();
        let item_path = item.path();
        let Ok(meta) = fs::symlink_metadata(&item_path) else {
            continue;
        };
        let is_symlink = meta.file_type().is_symlink();
        let symlink_target = if is_symlink {
            fs::read_link(&item_path).ok()
        } else {
            None
        };
        let is_dir = meta.is_dir();
        entries.push(FsEntry {
            path: item_path,
            name,
            is_dir,
            is_symlink,
            symlink_target,
            size: if meta.is_file() { meta.len() } else { 0 },
            modified: meta.modified().ok(),
            is_parent: false,
        });
    }
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    if let Some(parent) = path.parent() {
        entries.insert(
            0,
            FsEntry {
                path: parent.to_path_buf(),
                name: "..".into(),
                is_dir: true,
                is_symlink: false,
                symlink_target: None,
                size: 0,
                modified: None,
                is_parent: true,
            },
        );
    }
    Ok(entries)
}

pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

pub fn expand_tilde(input: &str) -> PathBuf {
    if input == "~" {
        home_dir()
    } else if let Some(rest) = input.strip_prefix("~/") {
        home_dir().join(rest)
    } else {
        PathBuf::from(input)
    }
}

pub fn resolve_path(input: &str, base: &Path) -> PathBuf {
    let trimmed = input.trim();
    let path = expand_tilde(trimmed);
    let joined = if path.is_absolute() {
        path
    } else {
        base.join(path)
    };
    normalize_path(&joined)
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let absolute = path.is_absolute();
    let mut parts: Vec<OsString> = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => parts.push(prefix.as_os_str().to_owned()),
            Component::RootDir => {}
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(last) = parts.last() {
                    if last != ".." {
                        parts.pop();
                    } else {
                        parts.push(OsString::from(".."));
                    }
                } else if !absolute {
                    parts.push(OsString::from(".."));
                }
            }
            Component::Normal(part) => parts.push(part.to_owned()),
        }
    }
    let mut out = if absolute {
        PathBuf::from("/")
    } else {
        PathBuf::new()
    };
    for part in parts {
        out.push(part);
    }
    if out.as_os_str().is_empty() {
        if absolute {
            PathBuf::from("/")
        } else {
            PathBuf::from(".")
        }
    } else {
        out
    }
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else if value >= 10.0 {
        format!("{value:.1} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

pub fn short_size(bytes: u64) -> String {
    human_size(bytes)
}

pub fn format_time(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return String::new();
    };
    let secs = time
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as libc::time_t)
        .unwrap_or(0);
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe {
        libc::localtime_r(&secs, &mut tm);
    }
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_relative_and_absolute_paths() {
        assert_eq!(
            normalize_path(Path::new("/a/b/../c")),
            PathBuf::from("/a/c")
        );
        assert_eq!(normalize_path(Path::new("./a/./b")), PathBuf::from("a/b"));
        assert_eq!(normalize_path(Path::new("../a")), PathBuf::from("../a"));
        assert_eq!(normalize_path(Path::new("/")), PathBuf::from("/"));
    }

    #[test]
    fn sizes_are_human_readable() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(1023), "1023 B");
        assert_eq!(human_size(1024), "1.00 KiB");
        assert_eq!(human_size(10 * 1024), "10.0 KiB");
        assert_eq!(human_size(200 * 1024), "200 KiB");
    }

    #[test]
    fn resolves_paths_against_base() {
        let base = Path::new("/tmp/base");
        assert_eq!(resolve_path("x/y", base), PathBuf::from("/tmp/base/x/y"));
        assert_eq!(resolve_path("/abs/./x", base), PathBuf::from("/abs/x"));
    }
}
