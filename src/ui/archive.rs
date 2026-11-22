use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, ClickEvent, Context, ExternalPaths, MouseButton, MouseDownEvent, SharedString,
    Window, div, px, rgb, rgba, uniform_list,
};

use crate::app::AppView;
use crate::archive::Child;
use crate::columns::ICON_WIDTH;
use crate::fsops;
use crate::state::{DragArchiveKeys, DragFsPaths, EditTarget, Pane};
use crate::ui::drag::DragChip;
use crate::ui::widgets::*;

impl AppView {
    pub(crate) fn render_archive_pane(
        &self,
        children: Rc<Vec<Child>>,
        visible: Rc<Vec<usize>>,
        pane_width: f32,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = self.theme;
        let has_archive = self.archive.archive.is_some();
        let inner_width = if has_archive {
            self.archive_inner_width(pane_width)
        } else {
            pane_width
        };
        let focused = self.focused == Pane::Archive;
        let can_extract = has_archive && self.archive.listing.is_some();
        let can_test = can_extract;
        let can_delete = can_extract
            && self.archive.format.map(|f| f.can_modify()).unwrap_or(false)
            && !self.arc_action_keys().is_empty();

        let compact = pane_width < 340.0;
        let list_area: AnyElement = if !has_archive {
            self.render_archive_welcome(cx)
        } else if let Some(error) = &self.archive.error {
            self.empty_state("Could not read archive", error, vec![], true)
        } else if self.archive.loading && self.archive.listing.is_none() {
            self.empty_state("Reading archive…", "Listing entries", vec![], false)
        } else if self.archive.listing.is_none() {
            self.empty_state(
                "No entries",
                "The archive could not be listed",
                vec![],
                false,
            )
        } else if visible.is_empty() {
            let subtitle = if !self.archive.filter.is_empty() {
                "No entries match the filter"
            } else if self.archive.inner.is_empty() {
                "This archive is empty"
            } else {
                "This folder is empty"
            };
            self.empty_state("Nothing to show", subtitle, vec![], false)
        } else {
            let count = visible.len();
            div()
                .flex_1()
                .min_h(px(0.))
                .child(
                    uniform_list(
                        "archive-list",
                        count,
                        cx.processor({
                            let children = children.clone();
                            let visible = visible.clone();
                            move |this: &mut AppView, range, window, cx| {
                                this.render_archive_rows(&children, &visible, range, window, cx)
                            }
                        }),
                    )
                    .track_scroll(self.archive.scroll.clone())
                    .h_full()
                    .w_full(),
                )
                .into_any_element()
        };

        let max_scroll = (inner_width - pane_width).max(0.0);
        let offset = self.archive_hscroll_offset.clamp(0.0, max_scroll);
        let scrollbar = if has_archive {
            self.h_scrollbar(Pane::Archive, pane_width, inner_width, cx)
        } else {
            div().into_any_element()
        };

        div()
            .id("archive-pane")
            .group("archive-drop")
            .relative()
            .flex()
            .flex_col()
            .w(px(pane_width))
            .flex_none()
            .min_w(px(0.))
            .h_full()
            .on_drop::<ExternalPaths>(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                this.drop_external_on_archive(paths.paths().to_vec(), cx);
            }))
            .on_drop::<DragFsPaths>(cx.listener(|this, drag: &DragFsPaths, _, cx| {
                this.drop_fs_paths(drag.paths.clone(), cx);
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.open_context_menu(
                        Pane::Archive,
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
                        rgb(theme.amber)
                    } else {
                        rgb(theme.dim)
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(if focused {
                                rgb(theme.amber)
                            } else {
                                rgb(theme.muted)
                            })
                            .child("ARCHIVE"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .truncate()
                            .text_xs()
                            .text_color(rgb(theme.dim))
                            .child(
                                self.archive
                                    .archive
                                    .as_ref()
                                    .and_then(|p| p.file_name())
                                    .and_then(|n| n.to_str())
                                    .map(|s| format!("· {s}"))
                                    .unwrap_or_else(|| "· none".to_string()),
                            ),
                    )
                    .child(self.btn(
                        "archive-extract",
                        if compact { "Extract" } else { "Extract →" },
                        can_extract,
                        cx,
                        |this, window, cx| {
                            this.focused = Pane::Archive;
                            this.open_extract_modal(window, cx)
                        },
                    ))
                    .child(
                        self.btn("archive-delete", "Delete", can_delete, cx, |this, _, cx| {
                            this.focused = Pane::Archive;
                            this.request_delete(cx)
                        }),
                    )
                    .child(
                        self.btn("archive-test", "Test", can_test, cx, |this, _, cx| {
                            this.focused = Pane::Archive;
                            this.test_archive(cx)
                        }),
                    ),
            )
            .child(
                div()
                    .id("archive-hscroll")
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
                            .children(if has_archive {
                                Some(self.render_archive_columns(cx))
                            } else {
                                None
                            })
                            .child(list_area),
                    ),
            )
            .child(scrollbar)
            .children(if self.preview.is_some() {
                Some(self.render_preview_pane(cx))
            } else {
                None
            })
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .group_drag_over::<ExternalPaths>("archive-drop", move |style| {
                        style
                            .bg(rgba(theme.amber << 8 | 0x22))
                            .border_2()
                            .border_color(rgb(theme.amber))
                    })
                    .group_drag_over::<DragFsPaths>("archive-drop", move |style| {
                        style
                            .bg(rgba(theme.amber << 8 | 0x22))
                            .border_2()
                            .border_color(rgb(theme.amber))
                    }),
            )
            .into_any_element()
    }

    pub(crate) fn render_archive_rows(
        &self,
        children: &[Child],
        visible: &[usize],
        range: std::ops::Range<usize>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let theme = self.theme;
        let entries = self
            .archive
            .listing
            .as_ref()
            .map(|listing| listing.entries.as_slice())
            .unwrap_or(&[]);
        range
            .filter_map(|visible_index| {
                let child_index = *visible.get(visible_index)?;
                let child = children.get(child_index)?;
                let key = self.archive.key_for(&child.name);
                let selected = self.archive.selected.contains(&key);
                let active = self.archive.active == Some(visible_index);
                let focused = self.focused == Pane::Archive;
                let entry = child.entry.and_then(|index| entries.get(index));
                let size = entry.map(|e| e.size).unwrap_or(0);
                let packed = entry.and_then(|e| e.packed);
                let modified = entry
                    .and_then(|e| e.modified.as_deref())
                    .unwrap_or("")
                    .to_string();
                let method = if let Some(entry) = entry {
                    if entry.is_symlink {
                        "symlink".to_string()
                    } else if entry.is_dir {
                        String::new()
                    } else if entry.encrypted {
                        "encrypted".to_string()
                    } else {
                        entry.method.clone().unwrap_or_default()
                    }
                } else {
                    String::new()
                };
                let icon = if child.is_dir {
                    theme.accent
                } else if entry.map(|e| e.is_symlink).unwrap_or(false) {
                    theme.purple
                } else {
                    theme.amber
                };
                let drag_keys = if child.is_parent {
                    Vec::new()
                } else if self.archive.selected.contains(&key) {
                    let mut keys: Vec<String> = self.archive.selected.iter().cloned().collect();
                    keys.sort();
                    keys
                } else {
                    vec![key.clone()]
                };
                let name = if child.is_parent {
                    "..".to_string()
                } else if child.is_dir {
                    format!("{}/", child.name)
                } else if let Some(target) = entry.and_then(|e| e.symlink_target.as_deref()) {
                    format!("{} → {}", child.name, target)
                } else {
                    child.name.clone()
                };
                Some(
                    div()
                        .id(SharedString::from(format!("arc-row-{visible_index}")))
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
                        .when(!child.is_parent, move |row| {
                            row.on_drag(
                                DragArchiveKeys { keys: drag_keys },
                                move |drag, _, _, cx| {
                                    let label = format!("Extract {} item(s)", drag.keys.len());
                                    cx.new(move |_| DragChip { label, theme })
                                },
                            )
                        })
                        .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                            this.on_archive_row_click(visible_index, event, window, cx);
                        }))
                        .on_mouse_down(
                            MouseButton::Right,
                            cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                                this.on_archive_context(visible_index, event, window, cx);
                            }),
                        )
                        .child(
                            div()
                                .size(px(ICON_WIDTH))
                                .flex_none()
                                .rounded(px(2.))
                                .bg(rgb(icon)),
                        )
                        .child(match self.archive_columns.name {
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
                                .w(px(self.archive_columns.size))
                                .flex_none()
                                .text_right()
                                .text_color(rgb(theme.muted))
                                .child(if child.is_dir {
                                    String::new()
                                } else {
                                    fsops::short_size(size)
                                }),
                        )
                        .child(
                            div()
                                .w(px(self.archive_columns.packed))
                                .flex_none()
                                .text_right()
                                .text_color(rgb(theme.dim))
                                .child(match packed {
                                    Some(packed) if !child.is_dir => fsops::short_size(packed),
                                    _ => String::new(),
                                }),
                        )
                        .child(
                            div()
                                .w(px(self.archive_columns.modified))
                                .flex_none()
                                .text_color(rgb(theme.muted))
                                .child(modified),
                        )
                        .child(
                            div()
                                .w(px(self.archive_columns.method))
                                .flex_none()
                                .text_color(rgb(theme.dim))
                                .truncate()
                                .child(method),
                        )
                        .into_any_element(),
                )
            })
            .collect()
    }

    pub(crate) fn on_archive_row_click(
        &mut self,
        visible_index: usize,
        event: &ClickEvent,
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
        let children = self.arc_visible_children();
        let Some(child) = children.get(visible_index).cloned() else {
            return;
        };
        if child.is_parent {
            self.archive.active = Some(visible_index);
            self.archive.anchor = Some(visible_index);
            self.archive.selected.clear();
            if event.click_count() >= 2 || self.settings.single_click_open {
                self.archive_up(cx);
            } else {
                cx.notify();
            }
            return;
        }
        let modifiers = event.modifiers();
        let Some(key) = self.arc_visible_key(visible_index) else {
            return;
        };
        if modifiers.shift {
            let anchor = self.archive.anchor.unwrap_or(visible_index);
            self.select_range(anchor, visible_index, cx);
        } else if modifiers.control {
            if !self.archive.selected.remove(&key) {
                self.archive.selected.insert(key);
            }
            self.archive.active = Some(visible_index);
            self.archive.anchor = Some(visible_index);
        } else {
            self.archive.selected.clear();
            self.archive.selected.insert(key);
            self.archive.active = Some(visible_index);
            self.archive.anchor = Some(visible_index);
        }
        if !modifiers.shift
            && !modifiers.control
            && self.settings.auto_preview
            && let Some(child) = self.arc_visible_children().get(visible_index).cloned()
            && !child.is_parent
            && !child.is_dir
        {
            self.preview_archive_entry(&child, true, cx);
        }
        if event.click_count() >= 2 {
            let children = self.arc_visible_children();
            if let Some(child) = children.get(visible_index).cloned() {
                if child.is_dir || child.is_parent {
                    self.enter_archive_child(&child, cx);
                } else {
                    self.open_archive_active(cx);
                }
            }
        } else if !modifiers.shift && !modifiers.control && self.settings.single_click_open {
            let children = self.arc_visible_children();
            if let Some(child) = children.get(visible_index).cloned() {
                if child.is_dir {
                    self.enter_archive_child(&child, cx);
                } else {
                    cx.notify();
                }
            } else {
                cx.notify();
            }
        } else {
            cx.notify();
        }
    }

    pub(crate) fn render_archive_welcome(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        let mut recent = div().flex().flex_col().gap_1().w_full();
        for (index, path) in self.recent.iter().enumerate() {
            let path_for_click = path.clone();
            recent = recent.child(
                div()
                    .id(SharedString::from(format!("recent-{index}")))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .rounded(px(4.))
                    .cursor_pointer()
                    .hover(move |style| style.bg(rgb(theme.hover)))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.open_archive(path_for_click.clone(), cx);
                    }))
                    .child(
                        div()
                            .size(px(7.))
                            .flex_none()
                            .rounded(px(2.))
                            .bg(rgb(theme.amber)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .truncate()
                            .text_color(rgb(theme.text))
                            .child(
                                path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("archive")
                                    .to_string(),
                            ),
                    )
                    .child(
                        div()
                            .max_w(px(260.))
                            .truncate()
                            .text_xs()
                            .text_color(rgb(theme.dim))
                            .child(
                                path.parent()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_default(),
                            ),
                    ),
            );
        }
        self.empty_state(
            "No archive open",
            "Double-click an archive in Files, or press Ctrl+L to type a path",
            if self.recent.is_empty() {
                vec![]
            } else {
                vec![
                    div()
                        .text_xs()
                        .text_color(rgb(theme.dim))
                        .mt_3()
                        .child("RECENT ARCHIVES")
                        .into_any_element(),
                    recent.into_any_element(),
                ]
            },
            false,
        )
    }
}
