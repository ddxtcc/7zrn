use std::cmp::Reverse;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::fsops;

use super::{Format, Listing};

#[derive(Clone, Debug)]
pub(crate) struct ArchiveStats {
    pub(crate) path: PathBuf,
    pub(crate) format: Format,
    pub(crate) archive_size: u64,
    pub(crate) total_size: u64,
    pub(crate) file_count: usize,
    pub(crate) folder_count: usize,
    pub(crate) encrypted_count: usize,
    pub(crate) methods: String,
    pub(crate) password_held: bool,
    pub(crate) categories: Vec<SizeCategory>,
    pub(crate) largest: Vec<LargestEntry>,
}

#[derive(Clone, Debug)]
pub(crate) struct SizeCategory {
    pub(crate) label: &'static str,
    pub(crate) bytes: u64,
    pub(crate) share: f64,
}

#[derive(Clone, Debug)]
pub(crate) struct LargestEntry {
    pub(crate) name: String,
    pub(crate) bytes: u64,
    pub(crate) share: f64,
}

impl ArchiveStats {
    pub(crate) fn build(
        path: &Path,
        format: Format,
        listing: &Listing,
        password_held: bool,
    ) -> Self {
        let total_size = listing.total_size();
        let mut buckets = [0u64; 7];
        for entry in &listing.entries {
            if entry.is_dir {
                continue;
            }
            let index = category_index(&entry.display_path);
            buckets[index] += entry.size;
        }
        let labels = [
            "Images",
            "Video",
            "Audio",
            "Archives",
            "Documents",
            "Code",
            "Other",
        ];
        let mut categories: Vec<SizeCategory> = buckets
            .iter()
            .enumerate()
            .filter(|(_, bytes)| **bytes > 0)
            .map(|(index, bytes)| SizeCategory {
                label: labels[index],
                bytes: *bytes,
                share: share(*bytes, total_size),
            })
            .collect();
        categories.sort_by_key(|category| Reverse(category.bytes));
        let mut largest: Vec<LargestEntry> = listing
            .entries
            .iter()
            .filter(|entry| !entry.is_dir)
            .map(|entry| LargestEntry {
                name: entry.display_path.clone(),
                bytes: entry.size,
                share: share(entry.size, total_size),
            })
            .collect();
        largest.sort_by_key(|entry| Reverse(entry.bytes));
        largest.truncate(8);
        let mut methods = BTreeSet::new();
        for entry in &listing.entries {
            if let Some(method) = &entry.method
                && !method.is_empty()
            {
                methods.insert(method.clone());
            }
        }
        let methods = if methods.is_empty() {
            "unknown".to_string()
        } else {
            methods.into_iter().collect::<Vec<_>>().join(", ")
        };
        Self {
            path: path.to_path_buf(),
            format,
            archive_size: listing.archive_size,
            total_size,
            file_count: listing.file_count(),
            folder_count: listing.dir_count(),
            encrypted_count: listing
                .entries
                .iter()
                .filter(|entry| entry.encrypted)
                .count(),
            methods,
            password_held,
            categories,
            largest,
        }
    }

    pub(crate) fn compression_ratio(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            self.archive_size as f64 / self.total_size as f64 * 100.0
        }
    }

    pub(crate) fn saved_bytes(&self) -> u64 {
        self.total_size.saturating_sub(self.archive_size)
    }

    pub(crate) fn summary(&self) -> String {
        format!(
            "Path: {}\nFormat: {}\nArchive size: {}\nUncompressed: {}\nSaved: {}\nCompression ratio: {:.1}%\nFiles: {}\nFolders: {}\nEncrypted entries: {}\nMethods: {}\nPassword held in memory: {}",
            self.path.display(),
            self.format.label(),
            fsops::human_size(self.archive_size),
            fsops::human_size(self.total_size),
            fsops::human_size(self.saved_bytes()),
            self.compression_ratio(),
            self.file_count,
            self.folder_count,
            self.encrypted_count,
            self.methods,
            if self.password_held { "yes" } else { "no" }
        )
    }
}

fn share(bytes: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        bytes as f64 / total as f64 * 100.0
    }
}

fn category_index(path: &str) -> usize {
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if matches!(
        extension.as_str(),
        "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "webp"
            | "bmp"
            | "svg"
            | "ico"
            | "tif"
            | "tiff"
            | "avif"
            | "heic"
    ) {
        0
    } else if matches!(
        extension.as_str(),
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "m4v" | "mpg" | "mpeg"
    ) {
        1
    } else if matches!(
        extension.as_str(),
        "mp3" | "flac" | "ogg" | "wav" | "m4a" | "aac" | "opus"
    ) {
        2
    } else if matches!(
        extension.as_str(),
        "7z" | "zip" | "rar" | "tar" | "gz" | "bz2" | "xz" | "zst" | "tgz" | "deb" | "rpm" | "iso"
    ) {
        3
    } else if matches!(
        extension.as_str(),
        "pdf"
            | "doc"
            | "docx"
            | "odt"
            | "xls"
            | "xlsx"
            | "ppt"
            | "pptx"
            | "txt"
            | "md"
            | "json"
            | "xml"
            | "yaml"
            | "yml"
            | "toml"
            | "csv"
    ) {
        4
    } else if matches!(
        extension.as_str(),
        "rs" | "py"
            | "js"
            | "ts"
            | "tsx"
            | "jsx"
            | "go"
            | "java"
            | "c"
            | "h"
            | "cpp"
            | "hpp"
            | "cs"
            | "rb"
            | "php"
            | "lua"
            | "sh"
            | "bash"
            | "sql"
            | "swift"
            | "kt"
            | "html"
            | "css"
            | "scss"
    ) {
        5
    } else {
        6
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::Entry;

    fn entry(path: &str, size: u64) -> Entry {
        Entry {
            raw_path: path.into(),
            display_path: path.into(),
            is_dir: false,
            is_symlink: false,
            symlink_target: None,
            size,
            packed: Some(size / 2),
            modified: None,
            method: Some("LZMA2".into()),
            encrypted: false,
        }
    }

    #[test]
    fn groups_sizes_and_finds_largest() {
        let listing = Listing {
            entries: vec![
                entry("photo.png", 100),
                entry("clip.mp4", 300),
                entry("src/main.rs", 50),
                entry("notes.md", 50),
            ],
            archive_size: 250,
        };
        let stats = ArchiveStats::build(Path::new("/tmp/test.zip"), Format::Zip, &listing, false);
        assert_eq!(stats.total_size, 500);
        assert_eq!(stats.file_count, 4);
        assert_eq!(stats.categories[0].label, "Video");
        assert!((stats.categories[0].share - 60.0).abs() < 0.01);
        assert_eq!(stats.largest[0].name, "clip.mp4");
        assert_eq!(stats.saved_bytes(), 250);
        assert_eq!(stats.compression_ratio(), 50.0);
    }
}
