use gpui::prelude::*;
use gpui::{
    AnyElement, ClickEvent, Context, MouseButton, MouseDownEvent, Window, deferred, div, px, rgb,
};

use crate::app::AppView;
use crate::settings::Setting;
use crate::state::Pane;

impl AppView {
    pub(crate) fn render_context_menu(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let menu = self.context_menu.as_ref()?;
        if self.modal.is_some() {
            return None;
        }
        let theme = self.theme;
        let width = 224.0;
        let rows = match menu.pane {
            Pane::Fs => 11.0,
            Pane::Archive => 16.0,
        };
        let height = rows * 26.0 + (rows - 1.0) * 2.0 + 8.0;
        let window_width = f32::from(window.bounds().size.width);
        let window_height = f32::from(window.bounds().size.height);
        let x = menu.x.clamp(4.0, (window_width - width - 8.0).max(4.0));
        let y = menu.y.clamp(4.0, (window_height - height - 8.0).max(4.0));
        let items: AnyElement = match menu.pane {
            Pane::Fs => {
                let active = self
                    .fs
                    .active
                    .and_then(|index| self.fs.visible().get(index).copied())
                    .and_then(|index| self.fs.entries.get(index));
                let active_any = active.is_some();
                let active_file = active
                    .map(|entry| !entry.is_parent && !entry.is_dir)
                    .unwrap_or(false);
                let can_new = !self.fs_action_paths().is_empty();
                let can_add = self.archive.archive.is_some()
                    && self.archive.listing.is_some()
                    && self
                        .archive
                        .format
                        .map(|format| format.can_modify())
                        .unwrap_or(false)
                    && !self.fs_action_paths().is_empty();
                let can_copy = active.is_some() || !self.fs.selected.is_empty();
                let can_checksum = !self.fs_action_paths().is_empty();
                let can_verify = self.can_verify_active();
                let can_merge = self.can_merge_volumes();
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(self.context_row(
                        "ctx-fs-open",
                        "Open",
                        active_any,
                        cx,
                        |this, _, cx| this.open_fs_active(cx),
                    ))
                    .child(self.context_row(
                        "ctx-fs-preview",
                        "Preview",
                        active_file,
                        cx,
                        |this, _, cx| this.preview_fs_active(cx),
                    ))
                    .child(self.context_row(
                        "ctx-fs-new",
                        "New archive…",
                        can_new,
                        cx,
                        |this, window, cx| this.open_new_archive_modal(window, cx),
                    ))
                    .child(self.context_row(
                        "ctx-fs-add",
                        "Add to archive",
                        can_add,
                        cx,
                        |this, _, cx| this.add_to_archive(cx),
                    ))
                    .child(self.context_row(
                        "ctx-fs-copy",
                        "Copy path(s)",
                        can_copy,
                        cx,
                        |this, _, cx| this.copy_selection(cx),
                    ))
                    .child(self.context_row(
                        "ctx-fs-move",
                        "Move to archive",
                        can_add,
                        cx,
                        |this, _, cx| this.move_to_archive(cx),
                    ))
                    .child(self.context_row(
                        "ctx-fs-checksums",
                        "Checksums…",
                        can_checksum,
                        cx,
                        |this, _, cx| this.checksum_fs_selection(cx),
                    ))
                    .child(self.context_row(
                        "ctx-fs-verify",
                        "Verify checksums…",
                        can_verify,
                        cx,
                        |this, _, cx| this.verify_checksums(cx),
                    ))
                    .child(self.context_row(
                        "ctx-fs-merge",
                        "Merge volumes",
                        can_merge,
                        cx,
                        |this, _, cx| this.merge_volumes(cx),
                    ))
                    .child(self.context_row(
                        "ctx-fs-hidden",
                        "Show hidden files",
                        true,
                        cx,
                        |this, _, cx| this.toggle_setting(Setting::ShowHidden, cx),
                    ))
                    .child(self.context_row(
                        "ctx-fs-refresh",
                        "Refresh",
                        true,
                        cx,
                        |this, _, cx| this.refresh_fs(cx),
                    ))
                    .into_any_element()
            }
            Pane::Archive => {
                let active = self
                    .archive
                    .active
                    .and_then(|index| self.arc_visible_children().get(index).cloned());
                let active_any = active.is_some();
                let active_file = active
                    .as_ref()
                    .map(|child| !child.is_parent && !child.is_dir)
                    .unwrap_or(false);
                let has_archive = self.archive.archive.is_some() && self.archive.listing.is_some();
                let can_delete = has_archive
                    && self
                        .archive
                        .format
                        .map(|format| format.can_modify())
                        .unwrap_or(false)
                    && !self.arc_action_keys().is_empty();
                let can_copy = active.is_some() || !self.archive.selected.is_empty();
                let can_edit = has_archive
                    && self
                        .archive
                        .format
                        .map(|format| format.can_modify())
                        .unwrap_or(false);
                let can_rename = can_edit
                    && active
                        .as_ref()
                        .map(|child| !child.is_parent)
                        .unwrap_or(false);
                let can_copy_move = can_edit && !self.arc_action_keys().is_empty();
                let can_checksum = has_archive && !self.arc_action_keys().is_empty();
                let can_move_out = can_edit && !self.arc_action_keys().is_empty();
                let can_verify = has_archive && self.can_verify_archive();
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(self.context_row(
                        "ctx-arc-open",
                        "Open",
                        active_any,
                        cx,
                        |this, _, cx| this.open_archive_active(cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-preview",
                        "Preview",
                        active_file,
                        cx,
                        |this, _, cx| this.preview_archive_active(cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-new-folder",
                        "New folder…",
                        can_edit,
                        cx,
                        |this, window, cx| this.open_new_folder_modal(window, cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-rename",
                        "Rename…",
                        can_rename,
                        cx,
                        |this, window, cx| this.open_rename_modal(window, cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-copy-to",
                        "Copy to folder…",
                        can_copy_move,
                        cx,
                        |this, window, cx| this.open_copy_move_modal(true, window, cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-move-to",
                        "Move to folder…",
                        can_copy_move,
                        cx,
                        |this, window, cx| this.open_copy_move_modal(false, window, cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-extract",
                        "Extract…",
                        has_archive,
                        cx,
                        |this, window, cx| this.open_extract_modal(window, cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-delete",
                        "Delete",
                        can_delete,
                        cx,
                        |this, _, cx| this.request_delete(cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-move-out",
                        "Move to Files",
                        can_move_out,
                        cx,
                        |this, _, cx| this.move_archive_to_fs(cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-checksums",
                        "Checksums…",
                        can_checksum,
                        cx,
                        |this, _, cx| this.checksum_archive_selection(cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-verify",
                        "Verify checksums…",
                        can_verify,
                        cx,
                        |this, _, cx| this.verify_archive_checksums(cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-convert",
                        "Convert…",
                        has_archive,
                        cx,
                        |this, window, cx| this.open_convert_modal(window, cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-test",
                        "Test archive",
                        has_archive,
                        cx,
                        |this, _, cx| this.test_archive(cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-copy",
                        "Copy path(s)",
                        can_copy,
                        cx,
                        |this, _, cx| {
                            this.focused = Pane::Archive;
                            this.copy_selection(cx);
                        },
                    ))
                    .child(self.context_row(
                        "ctx-arc-refresh",
                        "Refresh",
                        has_archive,
                        cx,
                        |this, _, cx| this.refresh_archive(cx),
                    ))
                    .child(self.context_row(
                        "ctx-arc-properties",
                        "Properties",
                        has_archive,
                        cx,
                        |this, _, cx| this.open_archive_properties(cx),
                    ))
                    .into_any_element()
            }
        };
        let panel = div()
            .absolute()
            .left(px(x))
            .top(px(y))
            .w(px(width))
            .p_1()
            .flex()
            .flex_col()
            .rounded(px(6.))
            .bg(rgb(theme.panel))
            .border_1()
            .border_color(rgb(theme.border2))
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(items);
        Some(
            deferred(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, _, cx| {
                            this.context_menu = None;
                            cx.notify();
                        }),
                    )
                    .child(panel),
            )
            .with_priority(9)
            .into_any_element(),
        )
    }

    fn context_row<F>(
        &self,
        id: &'static str,
        label: &'static str,
        enabled: bool,
        cx: &mut Context<Self>,
        action: F,
    ) -> AnyElement
    where
        F: Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    {
        let theme = self.theme;
        let mut row = div()
            .id(id)
            .h(px(26.))
            .px_2()
            .flex()
            .items_center()
            .rounded(px(3.))
            .text_sm()
            .text_color(rgb(if enabled { theme.text } else { theme.dim }))
            .child(label.to_string());
        if enabled {
            row = row
                .cursor_pointer()
                .hover(move |style| style.bg(rgb(theme.hover)))
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    this.context_menu = None;
                    action(this, window, cx);
                }));
        }
        row.into_any_element()
    }
}
