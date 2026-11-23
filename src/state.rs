use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use gpui::{ScrollStrategy, UniformListScrollHandle};

use crate::archive::{self, ArchiveEdit, ArchiveStats, Child, Format, NewFormat};
use crate::checksum::ChecksumEntry;
use crate::fsops::{self, FsEntry};
use crate::preview::PreviewContent;

#[derive(Clone, Debug)]
pub(crate) struct DragFsPaths {
    pub(crate) paths: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub(crate) struct DragArchiveKeys {
    pub(crate) keys: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pane {
    Fs,
    Archive,
}

impl Pane {
    pub(crate) fn other(self) -> Pane {
        match self {
            Pane::Fs => Pane::Archive,
            Pane::Archive => Pane::Fs,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EditTarget {
    FsPath,
    FsFilter,
    ArcFilter,
    NewName,
    NewPassword,
    NewSplit,
    NewExclude,
    ExtractDest,
    Password,
    ArchiveEdit,
    Search,
    Palette,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SearchScope {
    Fs,
    Archive,
}

#[derive(Clone, Debug)]
pub(crate) enum SearchHit {
    Fs {
        path: PathBuf,
        label: String,
        is_dir: bool,
    },
    Archive {
        key: String,
        label: String,
        is_dir: bool,
    },
}

pub(crate) struct EditState {
    pub(crate) target: EditTarget,
    pub(crate) text: String,
    pub(crate) cursor: usize,
}

pub(crate) struct FsState {
    pub(crate) path: PathBuf,
    pub(crate) entries: Vec<FsEntry>,
    pub(crate) selected: HashSet<PathBuf>,
    pub(crate) active: Option<usize>,
    pub(crate) anchor: Option<usize>,
    pub(crate) show_hidden: bool,
    pub(crate) filter: String,
    pub(crate) scroll: UniformListScrollHandle,
    pub(crate) history: Vec<PathBuf>,
    pub(crate) hist_pos: usize,
    pub(crate) error: Option<String>,
}

impl FsState {
    pub(crate) fn new(path: PathBuf) -> Self {
        let mut state = Self {
            path: path.clone(),
            entries: Vec::new(),
            selected: HashSet::new(),
            active: None,
            anchor: None,
            show_hidden: false,
            filter: String::new(),
            scroll: UniformListScrollHandle::new(),
            history: vec![path],
            hist_pos: 0,
            error: None,
        };
        state.reload();
        state
    }

    pub(crate) fn reload(&mut self) {
        match fsops::read_dir(&self.path) {
            Ok(entries) => {
                self.entries = entries;
                self.error = None;
            }
            Err(error) => {
                self.entries.clear();
                self.error = Some(error);
            }
        }
        self.clamp_active();
    }

    pub(crate) fn visible(&self) -> Vec<usize> {
        let needle = self.filter.to_lowercase();
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.is_parent
                    || ((self.show_hidden || !entry.name.starts_with('.'))
                        && (needle.is_empty() || entry.name.to_lowercase().contains(&needle)))
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub(crate) fn clamp_active(&mut self) {
        let len = self.visible().len();
        let start = if self.entries.first().is_some_and(|entry| entry.is_parent) && len > 1 {
            1
        } else {
            0
        };
        self.active = match self.active {
            Some(index) if index < len => Some(index),
            _ if len == 0 => None,
            _ => Some(start),
        };
        if self.anchor.is_none() {
            self.anchor = self.active;
        }
    }

    pub(crate) fn reset_active(&mut self) {
        let len = self.visible().len();
        let start = if self.entries.first().is_some_and(|entry| entry.is_parent) && len > 1 {
            1
        } else {
            0
        };
        self.active = if len == 0 { None } else { Some(start) };
        self.anchor = self.active;
        self.scroll.scroll_to_item(start, ScrollStrategy::Top);
    }

    pub(crate) fn selected_size(&self) -> u64 {
        self.entries
            .iter()
            .filter(|entry| !entry.is_parent && self.selected.contains(&entry.path))
            .map(|entry| entry.size)
            .sum()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ArchiveState {
    pub(crate) archive: Option<PathBuf>,
    pub(crate) format: Option<Format>,
    pub(crate) listing: Option<archive::Listing>,
    pub(crate) inner: String,
    pub(crate) selected: HashSet<String>,
    pub(crate) active: Option<usize>,
    pub(crate) anchor: Option<usize>,
    pub(crate) filter: String,
    pub(crate) scroll: UniformListScrollHandle,
    pub(crate) loading: bool,
    pub(crate) error: Option<String>,
    pub(crate) password: Option<String>,
}

impl ArchiveState {
    pub(crate) fn new() -> Self {
        Self {
            archive: None,
            format: None,
            listing: None,
            inner: String::new(),
            selected: HashSet::new(),
            active: None,
            anchor: None,
            filter: String::new(),
            scroll: UniformListScrollHandle::new(),
            loading: false,
            error: None,
            password: None,
        }
    }

    pub(crate) fn children(&self) -> Vec<Child> {
        self.listing
            .as_ref()
            .map(|listing| archive::children(&listing.entries, &self.inner))
            .unwrap_or_default()
    }

    pub(crate) fn visible_indices(&self, children: &[Child]) -> Vec<usize> {
        let needle = self.filter.to_lowercase();
        children
            .iter()
            .enumerate()
            .filter(|(_, child)| {
                child.is_parent || needle.is_empty() || child.name.to_lowercase().contains(&needle)
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub(crate) fn visible_children(&self) -> Vec<Child> {
        let children = self.children();
        let indices = self.visible_indices(&children);
        indices
            .into_iter()
            .map(|index| children[index].clone())
            .collect()
    }

    pub(crate) fn visible_len(&self) -> usize {
        self.visible_children().len()
    }

    pub(crate) fn key_for(&self, name: &str) -> String {
        if self.inner.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", self.inner, name)
        }
    }

    pub(crate) fn reset_active(&mut self) {
        let len = self.visible_len();
        let start = if !self.inner.is_empty() && len > 1 {
            1
        } else {
            0
        };
        self.active = if len == 0 { None } else { Some(start) };
        self.anchor = self.active;
        self.scroll.scroll_to_item(start, ScrollStrategy::Top);
    }

    pub(crate) fn clamp_active(&mut self) {
        let len = self.visible_len();
        let start = if !self.inner.is_empty() && len > 1 {
            1
        } else {
            0
        };
        self.active = match self.active {
            Some(index) if index < len => Some(index),
            _ if len == 0 => None,
            _ => Some(start),
        };
        if self.anchor.is_none() {
            self.anchor = self.active;
        }
    }

    pub(crate) fn selected_file_size(&self) -> u64 {
        let Some(listing) = &self.listing else {
            return 0;
        };
        listing
            .entries
            .iter()
            .filter(|entry| self.selected.contains(&entry.display_path) && !entry.is_dir)
            .map(|entry| entry.size)
            .sum()
    }

    pub(crate) fn expand_raw(&self, keys: &[String]) -> Vec<String> {
        let Some(listing) = &self.listing else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for key in keys {
            let prefix = format!("{key}/");
            for entry in &listing.entries {
                if entry.display_path == *key || entry.display_path.starts_with(&prefix) {
                    out.push(entry.raw_path.clone());
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }

    pub(crate) fn inner_exists(&self) -> bool {
        if self.inner.is_empty() {
            return true;
        }
        let Some(listing) = &self.listing else {
            return false;
        };
        let prefix = format!("{}/", self.inner);
        listing.entries.iter().any(|entry| {
            entry.display_path == self.inner || entry.display_path.starts_with(&prefix)
        })
    }

    pub(crate) fn fix_inner(&mut self) {
        while !self.inner.is_empty() && !self.inner_exists() {
            self.inner = archive::parent_inner(&self.inner);
        }
    }

    pub(crate) fn prune_selection(&mut self) {
        if let Some(listing) = &self.listing {
            let existing: HashSet<&str> = listing
                .entries
                .iter()
                .map(|entry| entry.display_path.as_str())
                .collect();
            self.selected.retain(|key| {
                existing.contains(key.as_str())
                    || listing
                        .entries
                        .iter()
                        .any(|entry| entry.display_path.starts_with(&format!("{key}/")))
            });
        }
    }

    pub(crate) fn clear(&mut self) {
        self.archive = None;
        self.format = None;
        self.listing = None;
        self.inner.clear();
        self.selected.clear();
        self.active = None;
        self.anchor = None;
        self.filter.clear();
        self.loading = false;
        self.error = None;
        self.password = None;
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PendingAction {
    List {
        path: PathBuf,
    },
    Extract {
        archive: PathBuf,
        format: Format,
        dest: PathBuf,
        raw_names: Vec<String>,
        count: usize,
        names: Vec<String>,
    },
    Test {
        archive: PathBuf,
        format: Format,
    },
    Add {
        archive: PathBuf,
        format: Format,
        paths: Vec<PathBuf>,
    },
    Open {
        archive: PathBuf,
        format: Format,
        dest: PathBuf,
        raw_name: String,
        display_name: String,
    },
    Delete {
        archive: PathBuf,
        format: Format,
        raw_names: Vec<String>,
        display_names: Vec<String>,
    },
    Edit {
        archive: PathBuf,
        format: Format,
        edit: ArchiveEdit,
    },
    Checksums {
        source: ChecksumSource,
    },
    OpenNested {
        archive: PathBuf,
        format: Format,
        dest: PathBuf,
        raw_name: String,
        display_name: String,
    },
}

impl PendingAction {
    pub(crate) fn archive_path(&self) -> &Path {
        match self {
            PendingAction::List { path } => path,
            PendingAction::Extract { archive, .. } => archive,
            PendingAction::Test { archive, .. } => archive,
            PendingAction::Add { archive, .. } => archive,
            PendingAction::Open { archive, .. } => archive,
            PendingAction::Delete { archive, .. } => archive,
            PendingAction::Edit { archive, .. } => archive,
            PendingAction::OpenNested { archive, .. } => archive,
            PendingAction::Checksums {
                source: ChecksumSource::Archive { archive, .. },
            } => archive,
            PendingAction::Checksums {
                source: ChecksumSource::Fs { .. },
            } => Path::new(""),
        }
    }

    pub(crate) fn purpose(&self) -> &'static str {
        match self {
            PendingAction::List { .. } => "open this archive",
            PendingAction::Extract { .. } => "extract from this archive",
            PendingAction::Test { .. } => "test this archive",
            PendingAction::Add { .. } => "add files to this archive",
            PendingAction::Open { .. } => "open a file from this archive",
            PendingAction::Delete { .. } => "delete entries from this archive",
            PendingAction::Edit { .. } => "edit this archive",
            PendingAction::OpenNested { .. } => "open this archive",
            PendingAction::Checksums { .. } => "compute checksums",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ChecksumSource {
    Fs {
        paths: Vec<PathBuf>,
    },
    Archive {
        archive: PathBuf,
        format: Format,
        dest: PathBuf,
        raw_names: Vec<String>,
    },
}

pub(crate) enum ArchiveEditKind {
    Rename { from: String },
    Copy { keys: Vec<String> },
    Move { keys: Vec<String> },
    NewFolder,
}

pub(crate) enum Modal {
    NewArchive {
        format: NewFormat,
        names: Vec<OsString>,
        base: PathBuf,
        name: String,
        compression: u8,
        password: String,
        split: String,
        excludes: String,
        error: Option<String>,
    },
    Extract {
        dest: String,
        selected_only: bool,
        count_selected: usize,
        count_total: usize,
        error: Option<String>,
    },
    DeleteConfirm {
        raw_names: Vec<String>,
        display_names: Vec<String>,
    },
    ArchiveEdit {
        title: String,
        kind: ArchiveEditKind,
        value: String,
        error: Option<String>,
    },
    ArchiveProperties {
        stats: ArchiveStats,
    },
    Checksums {
        entries: Vec<ChecksumEntry>,
    },
    TestPassed {
        stats: ArchiveStats,
    },
    Search {
        scope: SearchScope,
        query: String,
        content: bool,
        results: Vec<SearchHit>,
        selected: usize,
        error: Option<String>,
    },
    Palette {
        query: String,
        selected: usize,
    },
    Info {
        title: String,
        body: String,
        error: bool,
    },
    Password {
        message: String,
        password: String,
        error: Option<String>,
        action: PendingAction,
    },
    Settings,
    Help,
}

pub(crate) struct Message {
    pub(crate) text: String,
    pub(crate) error: bool,
}

pub(crate) struct Tab {
    pub(crate) title: String,
    pub(crate) fs: Option<FsState>,
    pub(crate) archive: Option<ArchiveState>,
    pub(crate) focused: Pane,
}

impl Tab {
    pub(crate) fn empty() -> Self {
        Self {
            title: String::new(),
            fs: None,
            archive: None,
            focused: Pane::Fs,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PreviewSplitDrag {
    pub(crate) start_y: f32,
    pub(crate) start_ratio: f32,
    pub(crate) height: f32,
}

pub(crate) struct PreviewView {
    pub(crate) title: String,
    pub(crate) path: Option<PathBuf>,
    pub(crate) content: PreviewContent,
}

pub(crate) struct ContextMenu {
    pub(crate) pane: Pane,
    pub(crate) x: f32,
    pub(crate) y: f32,
}
