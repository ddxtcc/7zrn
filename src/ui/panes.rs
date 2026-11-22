use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, ClickEvent, Context, CursorStyle, ExternalPaths, MouseButton, MouseDownEvent,
    SharedString, Window, div, px, rgb, rgba, uniform_list,
};

use crate::app::AppView;
use crate::columns::{HScrollDrag, ICON_WIDTH};
use crate::fsops::{self, FsEntry};
use crate::settings::Setting;
use crate::state::{DragArchiveKeys, DragFsPaths, EditTarget, Pane};
use crate::ui::drag::DragChip;
use crate::ui::widgets::*;

impl AppView {
    pub(crate) fn render_fs_pane(
        &self,
        visible: Rc<Vec<usize>>,
        children: Rc<Vec<FsEntry>>,
        pane_width: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = self.theme;
        let inner_width = if visible.is_empty() {
            pane_width
        } else {
            self.fs_inner_width(pane_width)
        };
        let focused = self.focused == Pane::Fs;
        let can_open = self
            .fs_active_key()
            .map(|path| path.is_dir() || path.is_file())
            .unwrap_or(false);
        let can_new = !self.fs_action_paths().is_empty();
        let can_add = self.archive.archive.is_some()
            && self.archive.listing.is_some()
            && self.archive.format.map(|f| f.can_modify()).unwrap_or(false)
            && !self.fs_action_paths().is_empty();
        let compact = pane_width < 340.0;
        let hidden_label = if compact {
            if self.fs.show_hidden { "H:on" } else { "H:off" }
        } else if self.fs.show_hidden {
            "Hidden: on"
        } else {
            "Hidden: off"
        };

        let list_area: AnyElement = if let Some(error) = &self.fs.error {
            self.empty_state("Cannot read folder", error, vec![], true)
        } else if visible.is_empty() {
            let subtitle = if self.fs.filter.is_empty() {
                "This folder is empty"
            } else {
                "No entries match the filter"
            };
            self.empty_state("Nothing to show", subtitle, vec![], false)
        } else {
            let count = visible.len();
            div()
                .flex_1()
                .min_h(px(0.))
                .child(
                    uniform_list(
                        "fs-list",
                        count,
                        cx.processor({
                            let visible = visible.clone();
                            move |this: &mut AppView, range, window, cx| {
                                this.render_fs_rows(&visible, &children, range, window, cx)
                            }
                        }),
                    )
                    .track_scroll(self.fs.scroll.clone())
                    .h_full()
                    .w_full(),
                )
                .into_any_element()
        };

        let max_scroll = (inner_width - pane_width).max(0.0);
        let offset = self.fs_hscroll_offset.clamp(0.0, max_scroll);

        div()
            .id("fs-pane")
            .group("fs-drop")
            .relative()
            .flex()
            .flex_col()
            .w(px(pane_width))
            .flex_none()
            .min_w(px(0.))
            .h_full()
            .on_drop::<ExternalPaths>(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                this.drop_external_on_fs(paths.paths().to_vec(), cx);
            }))
            .on_drop::<DragArchiveKeys>(cx.listener(|this, drag: &DragArchiveKeys, _, cx| {
                this.drop_archive_keys(drag.keys.clone(), cx);
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.open_context_menu(
                        Pane::Fs,
                        f32::from(event.position.x),
                        f32::from(event.position.y),
                        cx,
                    );
                }),
            )
            .child(
                div()
                    .h(px(34.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .bg(rgb(theme.panel))
                    .border_b_1()
                    .border_color(rgb(theme.border))
                    .child(div().size(px(7.)).rounded(px(2.)).bg(if focused {
                        rgb(theme.accent)
                    } else {
                        rgb(theme.dim)
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(if focused {
                                rgb(theme.accent)
                            } else {
                                rgb(theme.muted)
                            })
                            .child("FILES"),
                    )
                    .child(div().flex_1())
                    .child(self.btn("fs-open", "Open", can_open, cx, |this, _, cx| {
                        this.open_fs_active(cx)
                    }))
                    .child(self.btn("fs-new", "New", can_new, cx, |this, window, cx| {
                        this.open_new_archive_modal(window, cx)
                    }))
                    .child(self.btn(
                        "fs-add",
                        if compact { "Add" } else { "Add →" },
                        can_add,
                        cx,
                        |this, _, cx| {
                            this.focused = Pane::Fs;
                            this.add_to_archive(cx)
                        },
                    ))
                    .child(
                        self.btn("fs-hidden", hidden_label, true, cx, |this, _, cx| {
                            this.toggle_setting(Setting::ShowHidden, cx)
                        }),
                    ),
            )
            .child(
                div()
                    .id("fs-hscroll")
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .child(
                        div()
                            .h_full()
                            .flex()
                            .flex_col()
                            .w(px(inner_width))
                            .ml(px(-offset))
                            .child(self.render_fs_columns(cx))
                            .child(list_area),
                    ),
            )
            .child(self.h_scrollbar(Pane::Fs, pane_width, inner_width, cx))
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .group_drag_over::<ExternalPaths>("fs-drop", move |style| {
                        style
                            .bg(rgba(theme.accent << 8 | 0x22))
                            .border_2()
                            .border_color(rgb(theme.accent))
                    })
                    .group_drag_over::<DragArchiveKeys>("fs-drop", move |style| {
                        style
                            .bg(rgba(theme.accent << 8 | 0x22))
                            .border_2()
                            .border_color(rgb(theme.accent))
                    }),
            )
            .into_any_element()
    }

    pub(crate) fn h_scrollbar(
        &self,
        pane: Pane,
        pane_width: f32,
        inner_width: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = self.theme;
        let max_scroll = (inner_width - pane_width).max(0.0);
        if max_scroll <= 0.5 {
            return div().into_any_element();
        }
        let offset = match pane {
            Pane::Fs => self.fs_hscroll_offset,
            Pane::Archive => self.archive_hscroll_offset,
        }
        .clamp(0.0, max_scroll);
        let thumb_width = (pane_width * (pane_width / inner_width)).clamp(24.0, pane_width);
        let thumb_left = (offset / max_scroll) * (pane_width - thumb_width);
        div()
            .h(px(12.))
            .w_full()
            .flex_none()
            .bg(rgb(theme.panel))
            .border_t_1()
            .border_color(rgb(theme.border))
            .relative()
            .child(
                div()
                    .id(SharedString::from(match pane {
                        Pane::Fs => "fs-hscroll-thumb",
                        Pane::Archive => "archive-hscroll-thumb",
                    }))
                    .absolute()
                    .top(px(2.))
                    .left(px(thumb_left))
                    .w(px(thumb_width))
                    .h(px(8.))
                    .rounded(px(4.))
                    .bg(rgb(theme.border2))
                    .cursor(CursorStyle::PointingHand)
                    .hover(move |style| style.bg(rgb(theme.muted)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                            this.begin_hscroll_drag(
                                HScrollDrag {
                                    pane,
                                    start_x: f32::from(event.position.x),
                                    start_offset: offset,
                                    max_scroll,
                                    track_width: pane_width,
                                    thumb_width,
                                },
                                cx,
                            );
                        }),
                    ),
            )
            .into_any_element()
    }

    pub(crate) fn render_fs_rows(
        &self,
        visible: &[usize],
        entries: &[FsEntry],
        range: std::ops::Range<usize>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let theme = self.theme;
        range
            .filter_map(|visible_index| {
                let entry_index = *visible.get(visible_index)?;
                let entry = entries.get(entry_index)?;
                let selected = self.fs.selected.contains(&entry.path);
                let active = self.fs.active == Some(visible_index);
                let focused = self.focused == Pane::Fs;
                let icon = if entry.is_symlink {
                    theme.purple
                } else if entry.is_dir {
                    theme.accent
                } else if entry.is_archive() {
                    theme.amber
                } else {
                    theme.dim
                };
                let drag_paths = if self.fs.selected.contains(&entry.path) {
                    let mut paths: Vec<std::path::PathBuf> =
                        self.fs.selected.iter().cloned().collect();
                    paths.sort();
                    paths
                } else {
                    vec![entry.path.clone()]
                };
                let name = if entry.is_symlink {
                    match &entry.symlink_target {
                        Some(target) => format!(
                            "{} → {}",
                            entry.name,
                            target
                                .file_name()
                                .unwrap_or(target.as_os_str())
                                .to_string_lossy()
                        ),
                        None => entry.name.clone(),
                    }
                } else {
                    entry.name.clone()
                };
                Some(
                    div()
                        .id(SharedString::from(format!("fs-row-{visible_index}")))
                        .h(px(ROW_HEIGHT))
                        .w_full()
                        .flex()
                        .items_center()
                        .px_2()
                        .gap_2()
                        .cursor_pointer()
                        .bg(if selected {
                            rgb(theme.select)
                        } else {
                            rgb(if active && focused {
                                theme.row_active
                            } else {
                                theme.panel2
                            })
                        })
                        .border_b_1()
                        .border_color(rgb(if selected {
                            theme.row_selected_border
                        } else {
                            theme.row_border
                        }))
                        .hover(move |style| {
                            style.bg(rgb(if selected {
                                theme.row_selected_hover
                            } else {
                                theme.hover
                            }))
                        })
                        .on_drag(DragFsPaths { paths: drag_paths }, move |drag, _, _, cx| {
                            let label = format!("Add {} item(s)", drag.paths.len());
                            cx.new(move |_| DragChip { label, theme })
                        })
                        .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                            this.on_fs_row_click(visible_index, event, window, cx);
                        }))
                        .on_mouse_down(
                            MouseButton::Right,
                            cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                                this.on_fs_context(visible_index, event, window, cx);
                            }),
                        )
                        .child(
                            div()
                                .size(px(ICON_WIDTH))
                                .flex_none()
                                .rounded(px(2.))
                                .bg(rgb(icon)),
                        )
                        .child(match self.fs_columns.name {
                            Some(width) => div().w(px(width)).flex_none().truncate().child(name),
                            None => div()
                                .flex_1()
                                .min_w(px(crate::columns::min_width(
                                    crate::columns::ColumnId::Name,
                                )))
                                .truncate()
                                .child(name),
                        })
                        .child(
                            div()
                                .w(px(self.fs_columns.size))
                                .flex_none()
                                .text_right()
                                .text_color(rgb(theme.muted))
                                .child(if entry.is_dir {
                                    String::new()
                                } else {
                                    fsops::short_size(entry.size)
                                }),
                        )
                        .child(
                            div()
                                .w(px(self.fs_columns.modified))
                                .flex_none()
                                .text_color(rgb(theme.muted))
                                .child(fsops::format_time(entry.modified)),
                        )
                        .child(
                            div()
                                .w(px(self.fs_columns.kind))
                                .flex_none()
                                .text_color(rgb(theme.dim))
                                .truncate()
                                .child(entry.type_label()),
                        )
                        .into_any_element(),
                )
            })
            .collect()
    }

    pub(crate) fn on_fs_row_click(
        &mut self,
        visible_index: usize,
        event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focused = Pane::Fs;
        self.focus_handle.focus(_window);
        if let Some(edit) = &self.edit
            && matches!(edit.target, EditTarget::FsFilter | EditTarget::ArcFilter)
        {
            self.edit = None;
        }
        let parent_path = self.fs.visible().get(visible_index).and_then(|index| {
            let entry = self.fs.entries.get(*index)?;
            entry.is_parent.then(|| entry.path.clone())
        });
        if let Some(parent) = parent_path {
            self.fs.active = Some(visible_index);
            self.fs.anchor = Some(visible_index);
            self.fs.selected.clear();
            if event.click_count() >= 2 || self.settings.single_click_open {
                self.navigate_fs(parent, true, cx);
            } else {
                cx.notify();
            }
            return;
        }
        let modifiers = event.modifiers();
        let Some(key) = self.fs_visible_key(visible_index) else {
            return;
        };
        if modifiers.shift {
            let anchor = self.fs.anchor.unwrap_or(visible_index);
            self.select_range(anchor, visible_index, cx);
        } else if modifiers.control {
            if !self.fs.selected.remove(&key) {
                self.fs.selected.insert(key);
            }
            self.fs.active = Some(visible_index);
            self.fs.anchor = Some(visible_index);
        } else {
            self.fs.selected.clear();
            self.fs.selected.insert(key);
            self.fs.active = Some(visible_index);
            self.fs.anchor = Some(visible_index);
        }
        if !modifiers.shift && !modifiers.control && self.settings.auto_preview {
            let visible = self.fs.visible();
            if let Some(entry) = visible
                .get(visible_index)
                .and_then(|index| self.fs.entries.get(*index))
                && !entry.is_dir
                && !entry.is_archive()
            {
                let path = entry.path.clone();
                let title = entry.name.clone();
                self.preview_fs_file(path, title, true, cx);
            }
        }
        if event.click_count() >= 2 {
            self.activate_active(cx);
        } else if !modifiers.shift && !modifiers.control && self.settings.single_click_open {
            let visible = self.fs.visible();
            let opens = visible
                .get(visible_index)
                .and_then(|index| self.fs.entries.get(*index))
                .map(|entry| entry.is_dir || entry.is_archive())
                .unwrap_or(false);
            if opens {
                self.activate_active(cx);
            } else {
                cx.notify();
            }
        } else {
            cx.notify();
        }
    }
}
