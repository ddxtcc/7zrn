use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use gpui::{Context, ScrollStrategy};

use crate::archive;
use crate::fsops;
use crate::state::Pane;

use super::AppView;

impl AppView {
    pub(crate) fn open_target(&mut self, target: PathBuf, cx: &mut Context<Self>) {
        if target.is_dir() {
            self.navigate_fs(target, true, cx);
        } else {
            let parent = target
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("/"));
            self.navigate_fs(parent.clone(), true, cx);
            if archive::is_archive_name(target.file_name().and_then(|n| n.to_str()).unwrap_or("")) {
                self.open_archive(target, cx);
            } else {
                self.select_fs_path(&target, cx);
                self.set_message(
                    format!("{} is not a directory or archive", target.display()),
                    true,
                );
                cx.notify();
            }
        }
    }

    pub(crate) fn select_fs_path(&mut self, path: &Path, cx: &mut Context<Self>) {
        let visible = self.fs.visible();
        for (visible_index, entry_index) in visible.into_iter().enumerate() {
            if self.fs.entries[entry_index].path == path {
                self.fs.active = Some(visible_index);
                self.fs.anchor = Some(visible_index);
                self.fs.selected.clear();
                self.fs.selected.insert(path.to_path_buf());
                self.fs
                    .scroll
                    .scroll_to_item(visible_index, ScrollStrategy::Center);
                break;
            }
        }
        cx.notify();
    }

    pub(crate) fn select_fs_paths(&mut self, paths: &[PathBuf], cx: &mut Context<Self>) {
        let wanted: HashSet<PathBuf> = paths.iter().cloned().collect();
        let visible = self.fs.visible();
        self.fs.selected.clear();
        let mut first = None;
        for (visible_index, entry_index) in visible.into_iter().enumerate() {
            let path = &self.fs.entries[entry_index].path;
            if wanted.contains(path) {
                self.fs.selected.insert(path.clone());
                first.get_or_insert(visible_index);
            }
        }
        self.fs.active = first;
        self.fs.anchor = first;
        if let Some(index) = first {
            self.fs.scroll.scroll_to_item(index, ScrollStrategy::Center);
        }
        self.focused = Pane::Fs;
        cx.notify();
    }

    pub(crate) fn reveal_extracted(
        &mut self,
        dest: PathBuf,
        names: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        let paths: Vec<PathBuf> = names
            .into_iter()
            .filter(|name| !name.is_empty())
            .map(|name| dest.join(name))
            .filter(|path| path.exists())
            .collect();
        let (folder, selected) = reveal_plan(&dest, &paths);
        self.navigate_fs(folder, true, cx);
        if !selected.is_empty() {
            self.select_fs_paths(&selected, cx);
        }
    }

    pub(crate) fn reveal_dropped(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        let paths: Vec<PathBuf> = paths.into_iter().filter(|path| path.exists()).collect();
        if paths.is_empty() {
            return;
        }
        if paths.len() == 1 && paths[0].is_dir() {
            self.navigate_fs(paths[0].clone(), true, cx);
            return;
        }
        let parents: HashSet<PathBuf> = paths
            .iter()
            .filter_map(|path| path.parent().map(Path::to_path_buf))
            .collect();
        let parent = if parents.len() == 1 {
            parents.into_iter().next()
        } else {
            paths[0].parent().map(Path::to_path_buf)
        };
        if let Some(parent) = parent {
            self.navigate_fs(parent, true, cx);
            self.select_fs_paths(&paths, cx);
        }
    }

    pub(crate) fn navigate_fs(&mut self, path: PathBuf, record: bool, cx: &mut Context<Self>) {
        let path = fsops::normalize_path(&path);
        match fsops::read_dir(&path) {
            Ok(entries) => {
                self.fs.path = path;
                self.fs.entries = entries;
                self.fs.selected.clear();
                self.fs.filter.clear();
                self.fs.error = None;
                self.fs.active = None;
                self.fs.anchor = None;
                if record {
                    self.fs.history.truncate(self.fs.hist_pos + 1);
                    self.fs.history.push(self.fs.path.clone());
                    self.fs.hist_pos = self.fs.history.len() - 1;
                }
                self.fs.reset_active();
            }
            Err(error) => {
                self.set_message(error.clone(), true);
                self.fs.error = Some(error);
            }
        }
        cx.notify();
    }

    pub(crate) fn refresh_fs(&mut self, cx: &mut Context<Self>) {
        let active_key = self
            .fs
            .visible()
            .get(self.fs.active.unwrap_or(usize::MAX))
            .map(|index| self.fs.entries[*index].path.clone());
        match fsops::read_dir(&self.fs.path) {
            Ok(entries) => {
                self.fs.entries = entries;
                self.fs.error = None;
                let visible = self.fs.visible();
                self.fs.active = active_key
                    .and_then(|key| {
                        visible
                            .iter()
                            .position(|entry_index| self.fs.entries[*entry_index].path == key)
                    })
                    .or(if visible.is_empty() { None } else { Some(0) });
                self.fs.anchor = self.fs.active;
                self.fs.selected.retain(|key| key.exists());
            }
            Err(error) => {
                self.fs.error = Some(error.clone());
                self.set_message(error, true);
            }
        }
        cx.notify();
    }

    pub(crate) fn fs_up(&mut self, cx: &mut Context<Self>) {
        if let Some(parent) = self.fs.path.parent() {
            let parent = parent.to_path_buf();
            self.navigate_fs(parent, true, cx);
        }
    }

    pub(crate) fn fs_back(&mut self, cx: &mut Context<Self>) {
        if self.fs.hist_pos == 0 {
            return;
        }
        let target = self.fs.history[self.fs.hist_pos - 1].clone();
        match fsops::read_dir(&target) {
            Ok(entries) => {
                self.fs.hist_pos -= 1;
                self.fs.path = target;
                self.fs.entries = entries;
                self.fs.selected.clear();
                self.fs.filter.clear();
                self.fs.error = None;
                self.fs.active = None;
                self.fs.anchor = None;
                self.fs.reset_active();
            }
            Err(error) => {
                self.set_message(error, true);
            }
        }
        cx.notify();
    }

    pub(crate) fn fs_forward(&mut self, cx: &mut Context<Self>) {
        if self.fs.hist_pos + 1 >= self.fs.history.len() {
            return;
        }
        let target = self.fs.history[self.fs.hist_pos + 1].clone();
        match fsops::read_dir(&target) {
            Ok(entries) => {
                self.fs.hist_pos += 1;
                self.fs.path = target;
                self.fs.entries = entries;
                self.fs.selected.clear();
                self.fs.filter.clear();
                self.fs.error = None;
                self.fs.active = None;
                self.fs.anchor = None;
                self.fs.reset_active();
            }
            Err(error) => {
                self.set_message(error, true);
            }
        }
        cx.notify();
    }

    pub(crate) fn archive_up(&mut self, cx: &mut Context<Self>) {
        if self.archive.inner.is_empty() {
            return;
        }
        self.archive.inner = archive::parent_inner(&self.archive.inner);
        self.archive.selected.clear();
        self.archive.filter.clear();
        self.archive.reset_active();
        cx.notify();
    }

    pub(crate) fn go_back(&mut self, cx: &mut Context<Self>) {
        if self.focused == Pane::Fs {
            self.fs_back(cx);
        }
    }

    pub(crate) fn go_forward(&mut self, cx: &mut Context<Self>) {
        if self.focused == Pane::Fs {
            self.fs_forward(cx);
        }
    }

    pub(crate) fn go_up(&mut self, cx: &mut Context<Self>) {
        match self.focused {
            Pane::Fs => self.fs_up(cx),
            Pane::Archive => self.archive_up(cx),
        }
    }

    pub(crate) fn can_go_back(&self) -> bool {
        self.focused == Pane::Fs && self.fs.hist_pos > 0
    }

    pub(crate) fn can_go_forward(&self) -> bool {
        self.focused == Pane::Fs && self.fs.hist_pos + 1 < self.fs.history.len()
    }

    pub(crate) fn can_go_up(&self) -> bool {
        match self.focused {
            Pane::Fs => self.fs.path.parent().is_some(),
            Pane::Archive => !self.archive.inner.is_empty(),
        }
    }
}

fn reveal_plan(dest: &Path, paths: &[PathBuf]) -> (PathBuf, Vec<PathBuf>) {
    if paths.is_empty() {
        return (dest.to_path_buf(), Vec::new());
    }
    if paths.len() == 1 {
        let path = &paths[0];
        let folder = if path.is_dir() {
            dest.to_path_buf()
        } else {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| dest.to_path_buf())
        };
        return (folder, paths.to_vec());
    }
    let parents: HashSet<PathBuf> = paths
        .iter()
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .collect();
    if parents.len() == 1 {
        return (
            parents
                .into_iter()
                .next()
                .unwrap_or_else(|| dest.to_path_buf()),
            paths.to_vec(),
        );
    }
    let mut roots: Vec<PathBuf> = paths
        .iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(dest).ok()?;
            match relative.components().next()? {
                Component::Normal(name) => Some(dest.join(name)),
                _ => None,
            }
        })
        .collect();
    roots.sort();
    roots.dedup();
    (dest.to_path_buf(), roots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_plan_targets_containing_directory() {
        let dest = std::env::temp_dir().join(format!(
            "7zrn-reveal-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let app = dest.join("src/app");
        let other = dest.join("other");
        std::fs::create_dir_all(&app).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        let foo = app.join("foo.txt");
        let bar = app.join("bar.txt");
        std::fs::write(&foo, b"foo").unwrap();
        std::fs::write(&bar, b"bar").unwrap();

        let (folder, selected) = reveal_plan(&dest, std::slice::from_ref(&foo));
        assert_eq!(folder, app);
        assert_eq!(selected, vec![foo.clone()]);

        let (folder, selected) = reveal_plan(&dest, std::slice::from_ref(&dest.join("src")));
        assert_eq!(folder, dest);
        assert_eq!(selected, vec![dest.join("src")]);

        let (folder, selected) = reveal_plan(&dest, &[foo.clone(), bar.clone()]);
        assert_eq!(folder, app);
        assert_eq!(selected, vec![foo, bar]);

        let (folder, selected) = reveal_plan(&dest, &[app.join("foo.txt"), other.join("x.txt")]);
        assert_eq!(folder, dest);
        assert_eq!(selected, vec![dest.join("other"), dest.join("src")]);

        std::fs::remove_dir_all(dest).unwrap();
    }
}
