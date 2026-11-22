use gpui::{Context, KeyDownEvent, Window};

use crate::archive;
use crate::fsops;
use crate::settings::Setting;
use crate::state::*;
use crate::text::{next_boundary, prev_boundary};

use super::AppView;

impl AppView {
    pub(crate) fn begin_edit(
        &mut self,
        target: EditTarget,
        text: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match target {
            EditTarget::FsFilter | EditTarget::FsPath => self.focused = Pane::Fs,
            EditTarget::ArcFilter => self.focused = Pane::Archive,
            _ => {}
        }
        if let Some(edit) = &self.edit
            && edit.target == target
        {
            self.focus_handle.focus(window);
            cx.notify();
            return;
        }
        let cursor = text.len();
        self.edit = Some(EditState {
            target,
            text,
            cursor,
        });
        self.focus_handle.focus(window);
        cx.notify();
    }

    pub(crate) fn begin_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.focused {
            Pane::Fs => {
                let text = self.fs.filter.clone();
                self.begin_edit(EditTarget::FsFilter, text, window, cx);
            }
            Pane::Archive => {
                if self.archive.archive.is_some() {
                    let text = self.archive.filter.clone();
                    self.begin_edit(EditTarget::ArcFilter, text, window, cx);
                } else {
                    self.set_message("Open an archive first", true);
                    cx.notify();
                }
            }
        }
    }

    pub(crate) fn begin_path_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focused = Pane::Fs;
        let text = self.fs.path.display().to_string();
        self.begin_edit(EditTarget::FsPath, text, window, cx);
    }

    pub(crate) fn sync_filter_from_edit(&mut self, cx: &mut Context<Self>) {
        let mut search = false;
        if let Some(edit) = &self.edit {
            match edit.target {
                EditTarget::FsFilter => {
                    self.fs.filter = edit.text.clone();
                    self.fs.reset_active();
                }
                EditTarget::ArcFilter => {
                    self.archive.filter = edit.text.clone();
                    self.archive.reset_active();
                }
                EditTarget::Search => {
                    search = true;
                }
                EditTarget::Palette => {
                    if let Some(Modal::Palette { query, selected }) = &mut self.modal {
                        *query = edit.text.clone();
                        *selected = 0;
                    }
                }
                _ => {}
            }
        }
        if search {
            self.run_search(cx);
        } else {
            cx.notify();
        }
    }

    pub(crate) fn write_edit_text_to_target(&mut self, target: EditTarget, text: String) {
        match target {
            EditTarget::NewName => {
                if let Some(Modal::NewArchive { name, .. }) = &mut self.modal {
                    *name = text;
                }
            }
            EditTarget::NewPassword => {
                if let Some(Modal::NewArchive { password, .. }) = &mut self.modal {
                    *password = text;
                }
            }
            EditTarget::NewSplit => {
                if let Some(Modal::NewArchive { split, .. }) = &mut self.modal {
                    *split = text;
                }
            }
            EditTarget::NewExclude => {
                if let Some(Modal::NewArchive { excludes, .. }) = &mut self.modal {
                    *excludes = text;
                }
            }
            EditTarget::ExtractDest => {
                if let Some(Modal::Extract { dest, .. }) = &mut self.modal {
                    *dest = text;
                }
            }
            EditTarget::Password => {
                if let Some(Modal::Password { password, .. }) = &mut self.modal {
                    *password = text;
                }
            }
            EditTarget::ArchiveEdit => {
                if let Some(Modal::ArchiveEdit { value, .. }) = &mut self.modal {
                    *value = text;
                }
            }
            EditTarget::Search => {
                if let Some(Modal::Search { query, .. }) = &mut self.modal {
                    *query = text;
                }
            }
            EditTarget::Palette => {
                if let Some(Modal::Palette { query, .. }) = &mut self.modal {
                    *query = text;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn handle_edit_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let key = event.keystroke.key.as_str();
        let control = event.keystroke.modifiers.control;
        let alt = event.keystroke.modifiers.alt;

        if key == "escape" {
            if self.modal.is_some() {
                self.close_modal(cx);
                return true;
            }
            let Some(edit) = self.edit.take() else {
                return true;
            };
            match edit.target {
                EditTarget::FsFilter => {
                    self.fs.filter.clear();
                    self.fs.reset_active();
                }
                EditTarget::ArcFilter => {
                    self.archive.filter.clear();
                    self.archive.reset_active();
                }
                EditTarget::NewName
                | EditTarget::NewPassword
                | EditTarget::NewSplit
                | EditTarget::NewExclude
                | EditTarget::ExtractDest
                | EditTarget::Password
                | EditTarget::ArchiveEdit
                | EditTarget::Search
                | EditTarget::Palette => {
                    self.write_edit_text_to_target(edit.target, edit.text);
                }
                EditTarget::FsPath => {}
            }
            cx.notify();
            return true;
        }

        if key == "enter" {
            self.commit_edit(window, cx);
            return true;
        }
        if key == "tab" && self.modal.is_some() {
            self.focus_modal_field(window, cx);
            return true;
        }
        let selection_target = self.edit.as_ref().map(|edit| edit.target);
        if (key == "up" || key == "down")
            && matches!(
                selection_target,
                Some(EditTarget::Search | EditTarget::Palette)
            )
        {
            let delta = if key == "down" { 1 } else { -1 };
            if selection_target == Some(EditTarget::Search) {
                self.move_search_selection(delta, cx);
            } else {
                self.move_palette_selection(delta, cx);
            }
            return true;
        }

        let paste = if control && key == "v" {
            cx.read_from_clipboard().and_then(|item| item.text())
        } else {
            None
        };

        let mut changed = false;
        let mut filter_changed = false;
        if let Some(edit) = self.edit.as_mut() {
            match key {
                "backspace" => {
                    if edit.cursor > 0 {
                        let start = prev_boundary(&edit.text, edit.cursor);
                        edit.text.replace_range(start..edit.cursor, "");
                        edit.cursor = start;
                        changed = true;
                    }
                }
                "delete" => {
                    if edit.cursor < edit.text.len() {
                        let end = next_boundary(&edit.text, edit.cursor);
                        edit.text.replace_range(edit.cursor..end, "");
                        changed = true;
                    }
                }
                "left" => {
                    edit.cursor = prev_boundary(&edit.text, edit.cursor);
                }
                "right" => {
                    edit.cursor = next_boundary(&edit.text, edit.cursor);
                }
                "home" => {
                    edit.cursor = 0;
                }
                "end" => {
                    edit.cursor = edit.text.len();
                }
                "tab" => return true,
                _ => {
                    if control && key == "u" {
                        edit.text.clear();
                        edit.cursor = 0;
                        changed = true;
                    } else if control && key == "a" {
                        edit.cursor = edit.text.len();
                    } else if let Some(paste) = paste {
                        if edit.text.len() + paste.len() < 8192 {
                            let paste = paste.replace(['\n', '\r'], " ");
                            edit.text.insert_str(edit.cursor, &paste);
                            edit.cursor += paste.len();
                            changed = true;
                        }
                    } else if !control
                        && !alt
                        && let Some(ch) = &event.keystroke.key_char
                        && !ch.is_empty()
                        && !ch.chars().any(|c| c.is_control())
                        && edit.text.len() + ch.len() < 8192
                    {
                        edit.text.insert_str(edit.cursor, ch);
                        edit.cursor += ch.len();
                        changed = true;
                    }
                }
            }
            filter_changed =
                changed && matches!(edit.target, EditTarget::FsFilter | EditTarget::ArcFilter);
        }
        if changed {
            self.sync_filter_from_edit(cx);
            if !filter_changed {
                cx.notify();
            }
        }
        true
    }

    pub(crate) fn commit_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(edit) = self.edit.take() else { return };
        match edit.target {
            EditTarget::FsFilter => {
                self.fs.filter = edit.text;
                self.fs.reset_active();
                cx.notify();
            }
            EditTarget::ArcFilter => {
                self.archive.filter = edit.text;
                self.archive.reset_active();
                cx.notify();
            }
            EditTarget::FsPath => {
                self.navigate_fs_input(&edit.text, window, cx);
            }
            EditTarget::NewName
            | EditTarget::NewPassword
            | EditTarget::NewSplit
            | EditTarget::NewExclude
            | EditTarget::ExtractDest
            | EditTarget::Password
            | EditTarget::ArchiveEdit => {
                self.write_edit_text_to_target(edit.target, edit.text);
                self.submit_modal(window, cx);
            }
            EditTarget::Search => {
                self.write_edit_text_to_target(edit.target, edit.text);
                self.run_search(cx);
            }
            EditTarget::Palette => {
                self.write_edit_text_to_target(edit.target, edit.text);
                self.update_palette_selection();
                cx.notify();
            }
        }
    }

    pub(crate) fn navigate_fs_input(
        &mut self,
        input: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return;
        }
        let path = fsops::resolve_path(trimmed, &self.fs.path);
        if path.is_dir() {
            self.navigate_fs(path, true, cx);
        } else if path.is_file() {
            let parent = path.parent().map(|p| p.to_path_buf());
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if archive::is_archive_name(file_name) {
                if let Some(parent) = parent {
                    self.navigate_fs(parent.clone(), true, cx);
                }
                self.open_archive(path, cx);
            } else if let Some(parent) = parent {
                self.navigate_fs(parent, true, cx);
                self.select_fs_path(&path, cx);
            }
        } else {
            self.set_message(format!("No such path: {}", path.display()), true);
            self.edit = Some(EditState {
                target: EditTarget::FsPath,
                text: input.to_string(),
                cursor: input.len(),
            });
            self.focus_handle.focus(window);
            cx.notify();
        }
    }

    pub(crate) fn on_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let shortcut_key = event.keystroke.key.as_str();
        let shortcut_control = event.keystroke.modifiers.control;
        if self.context_menu.is_some() {
            self.context_menu = None;
            if shortcut_key == "escape" {
                cx.notify();
                return;
            }
        }
        if shortcut_control && self.modal.is_none() {
            if shortcut_key == "l" {
                if matches!(
                    self.edit.as_ref().map(|edit| edit.target),
                    Some(EditTarget::FsPath)
                ) {
                    self.focus_handle.focus(window);
                    cx.notify();
                } else {
                    self.edit = None;
                    self.begin_path_edit(window, cx);
                }
                return;
            }
            if shortcut_key == "f" {
                self.edit = None;
                self.begin_filter(window, cx);
                return;
            }
        }
        if self.edit.is_some() && self.handle_edit_key(event, window, cx) {
            return;
        }
        if self.modal.is_some() {
            if event.keystroke.modifiers.control && event.keystroke.key.as_str() == "c" {
                self.copy_modal_body(cx);
                return;
            }
            match event.keystroke.key.as_str() {
                "escape" => self.close_modal_with_escape(cx),
                "enter" => self.submit_modal(window, cx),
                "tab" => self.focus_modal_field(window, cx),
                _ => {}
            }
            return;
        }

        let key = event.keystroke.key.as_str();
        let control = event.keystroke.modifiers.control;
        let alt = event.keystroke.modifiers.alt;
        let shift = event.keystroke.modifiers.shift;

        if control && shift && key == "f" {
            self.open_search_modal(window, cx);
            return;
        }
        if control && shift && key == "p" {
            self.open_palette_modal(window, cx);
            return;
        }
        if control && shift && key == "t" {
            self.new_tab(cx);
            return;
        }
        if control && key == "w" {
            self.close_tab(self.active_tab, cx);
            return;
        }
        if control && key == "tab" {
            self.cycle_tab(!shift, cx);
            return;
        }
        if control && key == "q" {
            cx.quit();
            return;
        }
        if control && key == "h" {
            self.toggle_setting(Setting::ShowHidden, cx);
            return;
        }
        if control && key == "," {
            self.open_settings(cx);
            return;
        }
        if control && key == "a" {
            self.select_all(cx);
            return;
        }
        if control && key == "c" {
            self.copy_selection(cx);
            return;
        }
        if control && key == "n" {
            self.focused = Pane::Fs;
            self.open_new_archive_modal(window, cx);
            return;
        }
        if control && key == "e" {
            self.open_extract_modal(window, cx);
            return;
        }
        if control && key == "t" {
            self.test_archive(cx);
            return;
        }
        if control && key == "d" {
            self.focused = Pane::Fs;
            self.add_to_archive(cx);
            return;
        }
        if alt && key == "left" {
            self.go_back(cx);
            return;
        }
        if alt && key == "right" {
            self.go_forward(cx);
            return;
        }
        if alt && key == "up" {
            self.go_up(cx);
            return;
        }

        match key {
            "escape" => {
                if !self.fs.filter.is_empty() && self.focused == Pane::Fs {
                    self.fs.filter.clear();
                    self.fs.reset_active();
                    cx.notify();
                } else if !self.archive.filter.is_empty() && self.focused == Pane::Archive {
                    self.archive.filter.clear();
                    self.archive.reset_active();
                    cx.notify();
                } else {
                    self.clear_selection(cx);
                }
            }
            "tab" => {
                self.focused = self.focused.other();
                cx.notify();
            }
            "backspace" => self.go_up(cx),
            "enter" => self.activate_active(cx),
            "up" => self.move_active(-1, shift, cx),
            "down" => self.move_active(1, shift, cx),
            "pageup" => self.move_active(-10, shift, cx),
            "pagedown" => self.move_active(10, shift, cx),
            "home" => self.move_to_edge(false, shift, cx),
            "end" => self.move_to_edge(true, shift, cx),
            "space" => self.toggle_active(cx),
            "delete" => {
                if self.focused == Pane::Archive {
                    self.request_delete(cx);
                } else {
                    self.set_message("Delete is only available inside archives", false);
                    cx.notify();
                }
            }
            "f1" => {
                self.modal = Some(Modal::Help);
                self.edit = None;
                cx.notify();
            }
            "f3" => match self.focused {
                Pane::Fs => self.preview_fs_active(cx),
                Pane::Archive => self.preview_archive_active(cx),
            },
            "f2" => {
                if self.focused == Pane::Archive {
                    self.open_rename_modal(window, cx);
                }
            }
            "f5" => match self.focused {
                Pane::Fs => self.refresh_fs(cx),
                Pane::Archive => self.refresh_archive(cx),
            },
            "f7" => {
                if self.focused == Pane::Archive {
                    self.open_new_folder_modal(window, cx);
                }
            }
            _ => {}
        }
    }
}
