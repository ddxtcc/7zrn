use std::collections::BTreeMap;
use std::path::Path;

mod commands;
mod list;
mod stats;
#[cfg(test)]
mod tests;

pub use commands::*;
pub use list::*;
pub(crate) use stats::*;

pub const SEVEN_ZIP: &str = "7z";
pub const TAR: &str = "tar";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    SevenZ,
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    TarZst,
    Gzip,
    Bzip2,
    Xz,
    Zstd,
    Rar,
    Iso,
    Deb,
    Other,
}

impl Format {
    pub fn detect(path: &Path) -> Format {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            return Format::TarGz;
        }
        if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") || name.ends_with(".tbz") {
            return Format::TarBz2;
        }
        if name.ends_with(".tar.xz") || name.ends_with(".txz") {
            return Format::TarXz;
        }
        if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
            return Format::TarZst;
        }
        if name.ends_with(".7z.001") {
            return Format::SevenZ;
        }
        if name.ends_with(".zip.001") {
            return Format::Zip;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "7z" => Format::SevenZ,
            "zip" | "jar" | "war" | "ear" | "apk" | "xpi" | "ipa" | "epub" | "cbz" | "docx"
            | "xlsx" | "pptx" | "odt" | "ods" | "odp" | "aab" | "vsix" => Format::Zip,
            "tar" => Format::Tar,
            "gz" => Format::Gzip,
            "bz2" => Format::Bzip2,
            "xz" => Format::Xz,
            "zst" => Format::Zstd,
            "rar" | "r00" => Format::Rar,
            "iso" | "img" => Format::Iso,
            "deb" => Format::Deb,
            _ => Format::Other,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Format::SevenZ => "7Z",
            Format::Zip => "ZIP",
            Format::Tar => "TAR",
            Format::TarGz => "TAR.GZ",
            Format::TarBz2 => "TAR.BZ2",
            Format::TarXz => "TAR.XZ",
            Format::TarZst => "TAR.ZST",
            Format::Gzip => "GZ",
            Format::Bzip2 => "BZ2",
            Format::Xz => "XZ",
            Format::Zstd => "ZST",
            Format::Rar => "RAR",
            Format::Iso => "ISO",
            Format::Deb => "DEB",
            Format::Other => "ARCHIVE",
        }
    }

    pub fn is_tar_family(self) -> bool {
        matches!(
            self,
            Format::Tar | Format::TarGz | Format::TarBz2 | Format::TarXz | Format::TarZst
        )
    }

    pub fn is_compressed_tar(self) -> bool {
        matches!(
            self,
            Format::TarGz | Format::TarBz2 | Format::TarXz | Format::TarZst
        )
    }

    pub fn decompressor(self) -> Option<&'static str> {
        match self {
            Format::TarGz => Some("gzip"),
            Format::TarBz2 => Some("bzip2"),
            Format::TarXz => Some("xz"),
            Format::TarZst => Some("zstd"),
            _ => None,
        }
    }

    pub fn tar_suffix(self) -> Option<&'static str> {
        match self {
            Format::Tar => Some("tar"),
            Format::TarGz => Some("tar.gz"),
            Format::TarBz2 => Some("tar.bz2"),
            Format::TarXz => Some("tar.xz"),
            Format::TarZst => Some("tar.zst"),
            _ => None,
        }
    }

    pub fn can_modify(self) -> bool {
        matches!(self, Format::SevenZ | Format::Zip) || self.is_tar_family()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewFormat {
    SevenZ,
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    TarZst,
}

impl NewFormat {
    pub const ALL: [NewFormat; 7] = [
        NewFormat::SevenZ,
        NewFormat::Zip,
        NewFormat::Tar,
        NewFormat::TarGz,
        NewFormat::TarBz2,
        NewFormat::TarXz,
        NewFormat::TarZst,
    ];

    pub fn label(self) -> &'static str {
        match self {
            NewFormat::SevenZ => "7z",
            NewFormat::Zip => "zip",
            NewFormat::Tar => "tar",
            NewFormat::TarGz => "tar.gz",
            NewFormat::TarBz2 => "tar.bz2",
            NewFormat::TarXz => "tar.xz",
            NewFormat::TarZst => "tar.zst",
        }
    }

    pub fn ext(self) -> &'static str {
        self.label()
    }

    pub fn seven_zip_type(self) -> Option<&'static str> {
        match self {
            NewFormat::SevenZ => Some("7z"),
            NewFormat::Zip => Some("zip"),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub raw_path: String,
    pub display_path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub symlink_target: Option<String>,
    pub size: u64,
    pub packed: Option<u64>,
    pub modified: Option<String>,
    pub method: Option<String>,
    pub encrypted: bool,
}

#[derive(Clone, Debug)]
pub struct Listing {
    pub entries: Vec<Entry>,
    pub archive_size: u64,
}

impl Listing {
    pub fn total_size(&self) -> u64 {
        self.entries
            .iter()
            .filter(|e| !e.is_dir)
            .map(|e| e.size)
            .sum()
    }

    pub fn file_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.is_dir).count()
    }

    pub fn dir_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_dir).count()
    }
}
pub fn is_archive_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".tar.gz")
        || lower.ends_with(".tar.bz2")
        || lower.ends_with(".tar.xz")
        || lower.ends_with(".tar.zst")
        || lower.ends_with(".tgz")
        || lower.ends_with(".tbz")
        || lower.ends_with(".tbz2")
        || lower.ends_with(".txz")
        || lower.ends_with(".tzst")
        || lower.ends_with(".7z.001")
        || lower.ends_with(".zip.001")
    {
        return true;
    }
    let Some(ext) = lower.rsplit_once('.').map(|(_, e)| e) else {
        return false;
    };
    matches!(
        ext,
        "7z" | "zip"
            | "jar"
            | "war"
            | "ear"
            | "apk"
            | "xpi"
            | "ipa"
            | "epub"
            | "cbz"
            | "docx"
            | "xlsx"
            | "pptx"
            | "odt"
            | "ods"
            | "odp"
            | "aab"
            | "vsix"
            | "tar"
            | "gz"
            | "bz2"
            | "xz"
            | "zst"
            | "lz"
            | "lzma"
            | "rar"
            | "r00"
            | "iso"
            | "img"
            | "deb"
            | "rpm"
            | "cab"
            | "arj"
            | "lzh"
            | "lha"
            | "cpio"
            | "msi"
            | "dmg"
            | "vhd"
            | "wim"
    )
}
pub fn format_label(format: Format) -> &'static str {
    format.label()
}
pub fn password_required(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    (lower.contains("password")
        || lower.contains("encrypted")
        || lower.contains("break signaled")
        || lower.contains("enter password"))
        && !lower.contains("not a supported archive")
}
pub fn find_extracted(dest: &Path, display: &str) -> Option<std::path::PathBuf> {
    let relative = commands::safe_relative(display)?;
    let direct = dest.join(&relative);
    if is_regular_file(&direct) {
        return Some(direct);
    }
    let name = relative.file_name()?;
    fn walk(directory: &Path, name: &std::ffi::OsStr) -> Option<std::path::PathBuf> {
        let entries = std::fs::read_dir(directory).ok()?;
        let mut directories = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                directories.push(path);
            } else if meta.is_file() && path.file_name() == Some(name) {
                return Some(path);
            }
        }
        for directory in directories {
            if let Some(path) = walk(&directory, name) {
                return Some(path);
            }
        }
        None
    }
    walk(dest, name)
}

fn is_regular_file(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_file())
}

pub fn display_archive_path(raw: &str) -> String {
    let mut path = raw.trim_end_matches('/');
    while let Some(rest) = path.strip_prefix("./") {
        path = rest;
    }
    if path == "." {
        String::new()
    } else {
        path.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct Child {
    pub name: String,
    pub is_dir: bool,
    pub entry: Option<usize>,
    pub is_parent: bool,
}

pub fn children(entries: &[Entry], inner: &str) -> Vec<Child> {
    let prefix = if inner.is_empty() {
        String::new()
    } else {
        format!("{inner}/")
    };
    let mut map: BTreeMap<String, Child> = BTreeMap::new();
    for (index, entry) in entries.iter().enumerate() {
        let Some(rest) = entry.display_path.strip_prefix(&prefix) else {
            continue;
        };
        if rest.is_empty() {
            continue;
        }
        let (head, tail) = match rest.split_once('/') {
            Some((head, tail)) => (head, tail),
            None => (rest, ""),
        };
        if head.is_empty() {
            continue;
        }
        if tail.is_empty() {
            let is_dir = entry.is_dir;
            let child = map.entry(head.to_string()).or_insert_with(|| Child {
                name: head.to_string(),
                is_dir,
                entry: Some(index),
                is_parent: false,
            });
            if is_dir {
                child.is_dir = true;
                child.entry = Some(index);
            } else if child.entry.is_none() {
                child.entry = Some(index);
            }
        } else {
            let child = map.entry(head.to_string()).or_insert_with(|| Child {
                name: head.to_string(),
                is_dir: true,
                entry: None,
                is_parent: false,
            });
            child.is_dir = true;
        }
    }
    let mut children: Vec<Child> = map.into_values().collect();
    children.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    if !inner.is_empty() {
        children.insert(
            0,
            Child {
                name: "..".into(),
                is_dir: true,
                entry: None,
                is_parent: true,
            },
        );
    }
    children
}
pub fn parent_inner(inner: &str) -> String {
    match inner.rsplit_once('/') {
        Some((parent, _)) => parent.to_string(),
        None => String::new(),
    }
}

pub fn tar_extract_names(entries: &[Entry], keys: &[String]) -> Vec<String> {
    let top: Vec<&String> = keys
        .iter()
        .filter(|key| {
            !keys
                .iter()
                .any(|other| other != *key && key.starts_with(&format!("{other}/")))
        })
        .collect();
    let mut out = Vec::new();
    for key in top {
        if let Some(entry) = entries.iter().find(|entry| entry.display_path == *key) {
            out.push(entry.raw_path.clone());
            continue;
        }
        let prefix = format!("{key}/");
        let mut found = false;
        for entry in entries
            .iter()
            .filter(|entry| entry.display_path.starts_with(&prefix) && !entry.is_dir)
        {
            out.push(entry.raw_path.clone());
            found = true;
        }
        if !found {
            out.push(key.clone());
        }
    }
    out.sort();
    out.dedup();
    out
}
