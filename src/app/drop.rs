use std::path::PathBuf;

use gpui::Context;

use crate::archive;

use super::AppView;

impl AppView {
    pub(crate) fn drop_external_on_fs(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        self.reveal_dropped(paths, cx);
    }

    pub(crate) fn drop_external_on_archive(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        if self.archive.archive.is_some() {
            self.add_paths(paths, cx);
            return;
        }
        let Some(first) = paths.into_iter().next() else {
            return;
        };
        if first.is_dir() {
            self.navigate_fs(first, true, cx);
        } else if archive::is_archive_name(
            first
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(""),
        ) {
            self.open_archive(first, cx);
        } else {
            self.set_message("Open an archive first", true);
            cx.notify();
        }
    }

    pub(crate) fn drop_fs_paths(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        self.add_paths(paths, cx);
    }

    pub(crate) fn drop_archive_keys(&mut self, keys: Vec<String>, cx: &mut Context<Self>) {
        if self.archive.archive.is_none() {
            self.set_message("Open an archive first", true);
            cx.notify();
            return;
        }
        let dest = self.fs.path.clone();
        if let Err(message) = self.start_extract_keys(dest, keys, cx) {
            self.set_message(message, true);
            cx.notify();
        }
    }
}
