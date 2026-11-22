use gpui::{Context, MouseDownEvent, Window};

use crate::state::{ContextMenu, EditTarget, Pane};

use super::AppView;

impl AppView {
    pub(crate) fn open_context_menu(&mut self, pane: Pane, x: f32, y: f32, cx: &mut Context<Self>) {
        if self.modal.is_some() {
            return;
        }
        self.focused = pane;
        self.context_menu = Some(ContextMenu { pane, x, y });
        cx.notify();
    }

    pub(crate) fn on_fs_context(
        &mut self,
        visible_index: usize,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focused = Pane::Fs;
        self.focus_handle.focus(window);
        if let Some(edit) = &self.edit
            && matches!(edit.target, EditTarget::FsFilter | EditTarget::ArcFilter)
        {
            self.edit = None;
        }
        let entry = self.fs.visible().get(visible_index).and_then(|index| {
            self.fs
                .entries
                .get(*index)
                .map(|entry| (entry.path.clone(), entry.is_parent))
        });
        if let Some((path, is_parent)) = entry {
            self.fs.active = Some(visible_index);
            self.fs.anchor = Some(visible_index);
            if is_parent {
                self.fs.selected.clear();
            } else if !self.fs.selected.contains(&path) {
                self.fs.selected.clear();
                self.fs.selected.insert(path);
            }
        }
        self.open_context_menu(
            Pane::Fs,
            f32::from(event.position.x),
            f32::from(event.position.y),
            cx,
        );
    }

    pub(crate) fn on_archive_context(
        &mut self,
        visible_index: usize,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focused = Pane::Archive;
        self.focus_handle.focus(window);
        if let Some(edit) = &self.edit
            && matches!(edit.target, EditTarget::FsFilter | EditTarget::ArcFilter)
        {
            self.edit = None;
        }
        let child = self.arc_visible_children().get(visible_index).cloned();
        if let Some(child) = child {
            let key = self.archive.key_for(&child.name);
            self.archive.active = Some(visible_index);
            self.archive.anchor = Some(visible_index);
            if child.is_parent {
                self.archive.selected.clear();
            } else if !self.archive.selected.contains(&key) {
                self.archive.selected.clear();
                self.archive.selected.insert(key);
            }
        }
        self.open_context_menu(
            Pane::Archive,
            f32::from(event.position.x),
            f32::from(event.position.y),
            cx,
        );
    }
}
