use gpui::Context;

use crate::archive::Format;
use crate::fsops;
use crate::session::{Session, SessionTab};
use crate::state::{ArchiveState, FsState, Pane, Tab};

use super::AppView;

impl AppView {
    pub(crate) fn active_tab_title(&self) -> String {
        if let Some(archive) = &self.archive.archive {
            return archive
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Archive")
                .to_string();
        }
        self.fs
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| self.fs.path.display().to_string())
    }

    fn store_current(&mut self) {
        let title = self.active_tab_title();
        let focused = self.focused;
        let path = self.fs.path.clone();
        let fs = std::mem::replace(
            &mut self.fs,
            FsState::new_with_hidden(path, self.settings.show_hidden),
        );
        let archive = std::mem::replace(&mut self.archive, ArchiveState::new());
        let tab = &mut self.tabs[self.active_tab];
        tab.title = title;
        tab.focused = focused;
        tab.fs = Some(fs);
        tab.archive = Some(archive);
    }

    pub(crate) fn restore_session(&mut self, restore: bool, cx: &mut Context<Self>) {
        if !restore {
            return;
        }
        let session = Session::load();
        if session.tabs.is_empty() {
            return;
        }
        let mut tabs = Vec::new();
        for tab in &session.tabs {
            let fs_path = std::path::PathBuf::from(&tab.fs);
            if !fs_path.is_dir() {
                continue;
            }
            let mut archive = ArchiveState::new();
            if let Some(archive_path) = &tab.archive {
                let path = std::path::PathBuf::from(archive_path);
                if path.is_file() {
                    archive.archive = Some(path.clone());
                    archive.format = Some(Format::detect(&path));
                    archive.loading = true;
                }
            }
            tabs.push(Tab {
                title: fs_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Tab")
                    .to_string(),
                fs: Some(FsState::new_with_hidden(fs_path, self.settings.show_hidden)),
                archive: Some(archive),
                focused: Pane::Fs,
            });
        }
        if tabs.is_empty() {
            return;
        }
        let active = session.active.min(tabs.len() - 1);
        self.tabs = tabs;
        self.active_tab = active;
        let (fs, archive, focused) = {
            let tab = &mut self.tabs[active];
            (tab.fs.take(), tab.archive.take(), tab.focused)
        };
        if let Some(fs) = fs {
            self.fs = fs;
        }
        if let Some(archive) = archive {
            self.archive = archive;
        }
        self.focused = if self.archive.archive.is_some() {
            focused
        } else {
            Pane::Fs
        };
        if self.archive.archive.is_some() {
            self.refresh_archive(cx);
        }
    }

    pub(crate) fn save_session(&mut self) {
        self.store_current();
        let tabs = self
            .tabs
            .iter()
            .filter_map(|tab| {
                let fs = tab.fs.as_ref()?;
                Some(SessionTab {
                    fs: fs.path.display().to_string(),
                    archive: tab
                        .archive
                        .as_ref()
                        .and_then(|archive| archive.archive.as_ref())
                        .map(|path| path.display().to_string()),
                })
            })
            .collect();
        let _ = Session {
            active: self.active_tab,
            tabs,
        }
        .save();
    }

    pub(crate) fn new_tab(&mut self, cx: &mut Context<Self>) {
        if self.job.is_some() {
            self.set_message("Wait for the current operation to finish", true);
            cx.notify();
            return;
        }
        let path = self.fs.path.clone();
        self.store_current();
        self.fs = FsState::new_with_hidden(path, self.settings.show_hidden);
        self.archive = ArchiveState::new();
        self.tabs.push(Tab::empty());
        self.active_tab = self.tabs.len() - 1;
        self.focused = Pane::Fs;
        self.close_preview(cx);
        self.edit = None;
        self.modal = None;
        self.context_menu = None;
        cx.notify();
    }

    pub(crate) fn switch_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index == self.active_tab || index >= self.tabs.len() {
            return;
        }
        if self.job.is_some() {
            self.set_message("Wait for the current operation to finish", true);
            cx.notify();
            return;
        }
        let (next_fs, next_archive, next_focused) = {
            let tab = &mut self.tabs[index];
            (tab.fs.take(), tab.archive.take(), tab.focused)
        };
        self.store_current();
        self.fs = next_fs.unwrap_or_else(|| {
            FsState::new_with_hidden(fsops::home_dir(), self.settings.show_hidden)
        });
        self.archive = next_archive.unwrap_or_else(ArchiveState::new);
        self.active_tab = index;
        self.focused = if self.archive.archive.is_some() {
            next_focused
        } else {
            Pane::Fs
        };
        if self.archive.archive.is_some() && self.archive.listing.is_none() {
            self.refresh_archive(cx);
        }
        self.close_preview(cx);
        self.edit = None;
        self.modal = None;
        self.context_menu = None;
        cx.notify();
    }

    pub(crate) fn close_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        if self.job.is_some() {
            self.set_message("Wait for the current operation to finish", true);
            cx.notify();
            return;
        }
        if self.tabs.len() == 1 {
            self.archive.clear();
            self.fs = FsState::new_with_hidden(fsops::home_dir(), self.settings.show_hidden);
            self.close_preview(cx);
            self.edit = None;
            self.modal = None;
            self.context_menu = None;
            self.focused = Pane::Fs;
            cx.notify();
            return;
        }
        if index == self.active_tab {
            self.store_current();
            self.tabs.remove(index);
            let next = index.min(self.tabs.len() - 1);
            let (fs, archive, focused) = {
                let tab = &mut self.tabs[next];
                (tab.fs.take(), tab.archive.take(), tab.focused)
            };
            self.active_tab = next;
            self.fs = fs.unwrap_or_else(|| {
                FsState::new_with_hidden(fsops::home_dir(), self.settings.show_hidden)
            });
            self.archive = archive.unwrap_or_else(ArchiveState::new);
            self.focused = if self.archive.archive.is_some() {
                focused
            } else {
                Pane::Fs
            };
            if self.archive.archive.is_some() && self.archive.listing.is_none() {
                self.refresh_archive(cx);
            }
        } else {
            self.tabs.remove(index);
            if index < self.active_tab {
                self.active_tab -= 1;
            }
        }
        self.close_preview(cx);
        self.edit = None;
        self.modal = None;
        self.context_menu = None;
        cx.notify();
    }

    pub(crate) fn cycle_tab(&mut self, forward: bool, cx: &mut Context<Self>) {
        if self.tabs.len() < 2 {
            return;
        }
        let index = if forward {
            (self.active_tab + 1) % self.tabs.len()
        } else if self.active_tab == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab - 1
        };
        self.switch_tab(index, cx);
    }
}

impl Drop for AppView {
    fn drop(&mut self) {
        self.stop_audio();
        self.save_session();
    }
}
