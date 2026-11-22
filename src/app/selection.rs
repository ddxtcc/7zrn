use std::collections::HashSet;
use std::path::PathBuf;

use gpui::{ClipboardItem, Context, ScrollStrategy};

use crate::archive::Child;
use crate::state::Pane;

use super::AppView;

fn is_parent_key(key: &str) -> bool {
    key == ".." || key.rsplit('/').next() == Some("..")
}

impl AppView {
    pub(crate) fn fs_visible(&self) -> Vec<usize> {
        self.fs.visible()
    }

    pub(crate) fn fs_visible_key(&self, visible_index: usize) -> Option<PathBuf> {
        let visible = self.fs.visible();
        visible
            .get(visible_index)
            .map(|entry_index| self.fs.entries[*entry_index].path.clone())
    }

    pub(crate) fn fs_active_key(&self) -> Option<PathBuf> {
        self.fs.active.and_then(|index| self.fs_visible_key(index))
    }

    pub(crate) fn arc_visible_children(&self) -> Vec<Child> {
        self.archive.visible_children()
    }

    pub(crate) fn arc_visible_key(&self, visible_index: usize) -> Option<String> {
        self.arc_visible_children()
            .get(visible_index)
            .map(|child| self.archive.key_for(&child.name))
    }

    pub(crate) fn arc_active_key(&self) -> Option<String> {
        self.archive
            .active
            .and_then(|index| self.arc_visible_key(index))
    }

    pub(crate) fn fs_action_paths(&self) -> Vec<PathBuf> {
        let parent_path = self
            .fs
            .entries
            .iter()
            .find(|entry| entry.is_parent)
            .map(|entry| entry.path.clone());
        let paths: Vec<PathBuf> = if !self.fs.selected.is_empty() {
            let mut paths: Vec<PathBuf> = self.fs.selected.iter().cloned().collect();
            paths.sort();
            paths
        } else {
            self.fs_active_key().into_iter().collect()
        };
        paths
            .into_iter()
            .filter(|path| Some(path) != parent_path.as_ref())
            .collect()
    }

    pub(crate) fn arc_action_keys(&self) -> Vec<String> {
        let keys = if !self.archive.selected.is_empty() {
            let mut keys: Vec<String> = self.archive.selected.iter().cloned().collect();
            keys.sort();
            keys
        } else {
            self.arc_active_key().into_iter().collect()
        };
        keys.into_iter().filter(|key| !is_parent_key(key)).collect()
    }

    pub(crate) fn select_all(&mut self, cx: &mut Context<Self>) {
        match self.focused {
            Pane::Fs => {
                let keys: HashSet<PathBuf> = self
                    .fs_visible()
                    .into_iter()
                    .filter(|index| !self.fs.entries[*index].is_parent)
                    .map(|index| self.fs.entries[index].path.clone())
                    .collect();
                self.fs.selected = keys;
            }
            Pane::Archive => {
                let keys: HashSet<String> = self
                    .arc_visible_children()
                    .iter()
                    .filter(|child| !child.is_parent)
                    .map(|child| self.archive.key_for(&child.name))
                    .collect();
                self.archive.selected = keys;
            }
        }
        cx.notify();
    }

    pub(crate) fn clear_selection(&mut self, cx: &mut Context<Self>) {
        match self.focused {
            Pane::Fs => self.fs.selected.clear(),
            Pane::Archive => self.archive.selected.clear(),
        }
        cx.notify();
    }

    pub(crate) fn select_range(&mut self, from: usize, to: usize, cx: &mut Context<Self>) {
        let (start, end) = if from <= to { (from, to) } else { (to, from) };
        match self.focused {
            Pane::Fs => {
                let visible = self.fs.visible();
                let keys: HashSet<PathBuf> = (start..=end)
                    .filter_map(|index| visible.get(index))
                    .filter(|entry_index| !self.fs.entries[**entry_index].is_parent)
                    .map(|entry_index| self.fs.entries[*entry_index].path.clone())
                    .collect();
                self.fs.selected = keys;
                self.fs.active = Some(to);
                self.fs.anchor = Some(from);
                self.fs.scroll.scroll_to_item(to, ScrollStrategy::Center);
            }
            Pane::Archive => {
                let children = self.arc_visible_children();
                let keys: HashSet<String> = (start..=end)
                    .filter_map(|index| children.get(index))
                    .filter(|child| !child.is_parent)
                    .map(|child| self.archive.key_for(&child.name))
                    .collect();
                self.archive.selected = keys;
                self.archive.active = Some(to);
                self.archive.anchor = Some(from);
                self.archive
                    .scroll
                    .scroll_to_item(to, ScrollStrategy::Center);
            }
        }
        cx.notify();
    }

    pub(crate) fn move_active(&mut self, delta: isize, extend: bool, cx: &mut Context<Self>) {
        match self.focused {
            Pane::Fs => {
                let len = self.fs.visible().len();
                if len == 0 {
                    return;
                }
                let current = self.fs.active.unwrap_or(0) as isize;
                let next = (current + delta).clamp(0, len as isize - 1) as usize;
                if extend {
                    let anchor = self.fs.anchor.unwrap_or(self.fs.active.unwrap_or(next));
                    self.select_range(anchor, next, cx);
                } else {
                    self.fs.active = Some(next);
                    self.fs.anchor = Some(next);
                    self.fs.selected.clear();
                    if let Some(entry_index) = self.fs.visible().get(next).copied()
                        && !self.fs.entries[entry_index].is_parent
                    {
                        self.fs
                            .selected
                            .insert(self.fs.entries[entry_index].path.clone());
                    }
                    self.fs.scroll.scroll_to_item(next, ScrollStrategy::Center);
                }
            }
            Pane::Archive => {
                let len = self.archive.visible_len();
                if len == 0 {
                    return;
                }
                let current = self.archive.active.unwrap_or(0) as isize;
                let next = (current + delta).clamp(0, len as isize - 1) as usize;
                if extend {
                    let anchor = self
                        .archive
                        .anchor
                        .unwrap_or(self.archive.active.unwrap_or(next));
                    self.select_range(anchor, next, cx);
                } else {
                    self.archive.active = Some(next);
                    self.archive.anchor = Some(next);
                    self.archive.selected.clear();
                    let children = self.arc_visible_children();
                    if let Some(child) = children.get(next)
                        && !child.is_parent
                    {
                        self.archive
                            .selected
                            .insert(self.archive.key_for(&child.name));
                    }
                    self.archive
                        .scroll
                        .scroll_to_item(next, ScrollStrategy::Center);
                }
            }
        }
        cx.notify();
    }

    pub(crate) fn move_to_edge(&mut self, end: bool, extend: bool, cx: &mut Context<Self>) {
        let len = match self.focused {
            Pane::Fs => self.fs.visible().len(),
            Pane::Archive => self.archive.visible_len(),
        };
        if len == 0 {
            return;
        }
        let target = if end { len - 1 } else { 0 };
        let current = match self.focused {
            Pane::Fs => self.fs.active.unwrap_or(0),
            Pane::Archive => self.archive.active.unwrap_or(0),
        };
        if extend {
            let anchor = match self.focused {
                Pane::Fs => self.fs.anchor.unwrap_or(current),
                Pane::Archive => self.archive.anchor.unwrap_or(current),
            };
            self.select_range(anchor, target, cx);
        } else {
            self.move_active(target as isize - current as isize, false, cx);
        }
    }

    pub(crate) fn toggle_active(&mut self, cx: &mut Context<Self>) {
        match self.focused {
            Pane::Fs => {
                let parent = self
                    .fs
                    .visible()
                    .get(self.fs.active.unwrap_or(usize::MAX))
                    .is_some_and(|index| self.fs.entries[*index].is_parent);
                if parent {
                    return;
                }
                let Some(key) = self.fs_active_key() else {
                    return;
                };
                if !self.fs.selected.remove(&key) {
                    self.fs.selected.insert(key);
                }
                self.fs.anchor = self.fs.active;
                let len = self.fs.visible().len();
                if let Some(index) = self.fs.active
                    && index + 1 < len
                {
                    self.fs.active = Some(index + 1);
                    self.fs
                        .scroll
                        .scroll_to_item(index + 1, ScrollStrategy::Center);
                }
            }
            Pane::Archive => {
                let Some(index) = self.archive.active else {
                    return;
                };
                let parent = self
                    .arc_visible_children()
                    .get(index)
                    .is_some_and(|child| child.is_parent);
                if parent {
                    return;
                }
                let Some(key) = self.arc_active_key() else {
                    return;
                };
                if !self.archive.selected.remove(&key) {
                    self.archive.selected.insert(key);
                }
                self.archive.anchor = self.archive.active;
                let len = self.archive.visible_len();
                if let Some(index) = self.archive.active
                    && index + 1 < len
                {
                    self.archive.active = Some(index + 1);
                    self.archive
                        .scroll
                        .scroll_to_item(index + 1, ScrollStrategy::Center);
                }
            }
        }
        cx.notify();
    }

    pub(crate) fn copy_selection(&mut self, cx: &mut Context<Self>) {
        let text = match self.focused {
            Pane::Fs => {
                let paths = if self.fs.selected.is_empty() {
                    self.fs_active_key().into_iter().collect()
                } else {
                    let mut paths: Vec<PathBuf> = self.fs.selected.iter().cloned().collect();
                    paths.sort();
                    paths
                };
                paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            Pane::Archive => self.arc_action_keys().join("\n"),
        };
        if text.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.set_message("Copied path(s) to clipboard", false);
        cx.notify();
    }

    pub(crate) fn enter_archive_child(&mut self, child: &Child, cx: &mut Context<Self>) {
        if child.is_parent {
            self.archive_up(cx);
            return;
        }
        if !child.is_dir {
            return;
        }
        self.archive.inner = self.archive.key_for(&child.name);
        self.archive.filter.clear();
        self.archive.selected.clear();
        self.archive.reset_active();
        cx.notify();
    }

    pub(crate) fn activate_active(&mut self, cx: &mut Context<Self>) {
        match self.focused {
            Pane::Fs => {
                let Some(index) = self.fs.active else { return };
                let visible = self.fs.visible();
                let Some(entry_index) = visible.get(index).copied() else {
                    return;
                };
                let entry = &self.fs.entries[entry_index];
                if entry.is_dir {
                    let path = entry.path.clone();
                    self.navigate_fs(path, true, cx);
                } else if entry.is_archive() {
                    let path = entry.path.clone();
                    self.open_archive(path, cx);
                } else {
                    let path = entry.path.clone();
                    self.open_path_external(path, cx);
                }
            }
            Pane::Archive => {
                let Some(index) = self.archive.active else {
                    return;
                };
                let children = self.arc_visible_children();
                let Some(child) = children.get(index).cloned() else {
                    return;
                };
                if child.is_dir || child.is_parent {
                    self.enter_archive_child(&child, cx);
                } else {
                    self.open_archive_active(cx);
                }
            }
        }
    }

    pub(crate) fn open_fs_active(&mut self, cx: &mut Context<Self>) {
        self.focused = Pane::Fs;
        self.activate_active(cx);
    }
}
