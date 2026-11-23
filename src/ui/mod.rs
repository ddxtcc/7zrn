use std::path::{Component, PathBuf};
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, ClickEvent, Context, CursorStyle, IntoElement, MouseButton, MouseDownEvent, Render,
    SharedString, Window, div, px, rgb,
};

use crate::app::AppView;
use crate::app::layout::SPLITTER_WIDTH;
use crate::fsops;
use crate::job;
use crate::state::{EditTarget, Modal, Pane};
use crate::ui::widgets::*;

mod archive;
mod checksums;
mod code;
mod columns;
mod context;
mod drag;
mod help_modal;
mod markdown;
mod modals;
mod panes;
mod preview;
mod properties;
mod settings_modal;
mod tabs;
pub(crate) mod widgets;

impl AppView {
    pub(crate) fn render_top_bar(&self, window_width: f32, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        let compact = window_width < 760.0;
        let filter_width = if window_width < 900.0 { 120.0 } else { 190.0 };
        let can_back = self.can_go_back();
        let can_forward = self.can_go_forward();
        let can_up = self.can_go_up();
        let filter_target = match self.focused {
            Pane::Fs => Some(EditTarget::FsFilter),
            Pane::Archive => {
                if self.archive.archive.is_some() {
                    Some(EditTarget::ArcFilter)
                } else {
                    None
                }
            }
        };
        let (filter_text, filter_active, filter_cursor) = match (&self.edit, filter_target) {
            (Some(edit), Some(target)) if edit.target == target => {
                (edit.text.clone(), true, edit.cursor)
            }
            _ => {
                let text = match self.focused {
                    Pane::Fs => self.fs.filter.clone(),
                    Pane::Archive => self.archive.filter.clone(),
                };
                (text, false, 0)
            }
        };

        div()
            .h(px(42.))
            .flex_none()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .bg(rgb(theme.panel))
            .border_b_1()
            .border_color(rgb(theme.border))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_color(rgb(theme.accent))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_size(px(15.))
                            .child("7zrn"),
                    )
                    .child(div().w(px(1.)).h(px(18.)).bg(rgb(theme.border))),
            )
            .child(self.btn("nav-back", "←", can_back, cx, |this, _, cx| {
                this.go_back(cx)
            }))
            .child(
                self.btn("nav-forward", "→", can_forward, cx, |this, _, cx| {
                    this.go_forward(cx)
                }),
            )
            .child(self.btn("nav-up", "↑", can_up, cx, |this, _, cx| this.go_up(cx)))
            .child(self.render_location(cx))
            .child(self.input_field(
                InputField {
                    id: "filter-input",
                    text: &filter_text,
                    placeholder: "Filter",
                    active: filter_active,
                    cursor: filter_cursor,
                    width: Some(filter_width),
                    target: filter_target,
                    click_text: filter_text.clone(),
                },
                cx,
            ))
            .child(self.btn(
                "settings-button",
                if compact { "⚙" } else { "Settings" },
                true,
                cx,
                |this, _, cx| this.open_settings(cx),
            ))
            .child(self.btn("help-button", "?", true, cx, |this, _, cx| {
                this.modal = Some(Modal::Help);
                this.edit = None;
                cx.notify();
            }))
            .into_any_element()
    }

    pub(crate) fn render_location(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.focused {
            Pane::Fs => self.render_fs_breadcrumb(cx),
            Pane::Archive => self.render_archive_breadcrumb(cx),
        }
    }

    pub(crate) fn render_fs_breadcrumb(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        if let Some(edit) = &self.edit
            && edit.target == EditTarget::FsPath
        {
            let text = edit.text.clone();
            return self.input_field(
                InputField {
                    id: "fs-path-input",
                    text: &text,
                    placeholder: "Path",
                    active: true,
                    cursor: edit.cursor,
                    width: None,
                    target: Some(EditTarget::FsPath),
                    click_text: text.clone(),
                },
                cx,
            );
        }
        let mut crumbs: Vec<(String, PathBuf)> = vec![("/".to_string(), PathBuf::from("/"))];
        let mut accumulated = PathBuf::from("/");
        for component in self.fs.path.components() {
            if let Component::Normal(name) = component {
                accumulated.push(name);
                crumbs.push((name.to_string_lossy().into_owned(), accumulated.clone()));
            }
        }
        let mut row = div()
            .flex_1()
            .min_w(px(0.))
            .flex()
            .items_center()
            .gap_1()
            .overflow_hidden();
        for (index, (label, path)) in crumbs.into_iter().enumerate() {
            if index > 0 {
                row = row.child(div().text_color(rgb(theme.dim)).child("›"));
            }
            let path_for_click = path.clone();
            row = row.child(
                div()
                    .id(SharedString::from(format!("fs-crumb-{index}")))
                    .flex_none()
                    .max_w(px(190.))
                    .truncate()
                    .px_1()
                    .rounded(px(3.))
                    .text_color(if index == 0 {
                        rgb(theme.muted)
                    } else {
                        rgb(theme.text)
                    })
                    .cursor_pointer()
                    .hover(move |style| style.bg(rgb(theme.hover)))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.focused = Pane::Fs;
                        this.navigate_fs(path_for_click.clone(), true, cx);
                    }))
                    .child(label),
            );
        }
        row.child(
            div()
                .id("fs-edit-path")
                .flex_none()
                .px_1()
                .rounded(px(3.))
                .text_color(rgb(theme.dim))
                .cursor_pointer()
                .hover(move |style| style.bg(rgb(theme.hover)).text_color(rgb(theme.text)))
                .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                    this.begin_path_edit(window, cx);
                }))
                .child("✎"),
        )
        .into_any_element()
    }

    pub(crate) fn render_archive_breadcrumb(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        let Some(archive_path) = &self.archive.archive else {
            return div()
                .flex_1()
                .min_w(px(0.))
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_color(rgb(theme.dim))
                        .child("No archive open - double-click one in Files"),
                )
                .into_any_element();
        };
        let archive_name = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive")
            .to_string();
        let mut row = div()
            .flex_1()
            .min_w(px(0.))
            .flex()
            .items_center()
            .gap_1()
            .overflow_hidden()
            .child(
                div()
                    .id("archive-crumb-root")
                    .flex_none()
                    .max_w(px(220.))
                    .truncate()
                    .px_1()
                    .rounded(px(3.))
                    .text_color(rgb(theme.amber))
                    .cursor_pointer()
                    .hover(move |style| style.bg(rgb(theme.hover)))
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.focused = Pane::Archive;
                        this.archive.inner.clear();
                        this.archive.filter.clear();
                        this.archive.selected.clear();
                        this.archive.reset_active();
                        cx.notify();
                    }))
                    .child(archive_name),
            );
        if let Some(format) = self.archive.format {
            row = row.child(
                div()
                    .flex_none()
                    .text_color(rgb(theme.dim))
                    .child(format!("· {}", format.label())),
            );
        }
        if !self.archive.inner.is_empty() {
            let mut accumulated = String::new();
            for (index, part) in self.archive.inner.split('/').enumerate() {
                if !accumulated.is_empty() {
                    accumulated.push('/');
                }
                accumulated.push_str(part);
                let target = accumulated.clone();
                row = row
                    .child(div().text_color(rgb(theme.dim)).child("›"))
                    .child(
                        div()
                            .id(SharedString::from(format!("archive-crumb-{index}")))
                            .flex_none()
                            .max_w(px(160.))
                            .truncate()
                            .px_1()
                            .rounded(px(3.))
                            .text_color(rgb(theme.text))
                            .cursor_pointer()
                            .hover(move |style| style.bg(rgb(theme.hover)))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                this.focused = Pane::Archive;
                                this.archive.inner = target.clone();
                                this.archive.filter.clear();
                                this.archive.selected.clear();
                                this.archive.reset_active();
                                cx.notify();
                            }))
                            .child(part.to_string()),
                    );
            }
        }
        row.into_any_element()
    }

    pub(crate) fn render_splitter(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        div()
            .id("pane-splitter")
            .w(px(SPLITTER_WIDTH))
            .flex_none()
            .h_full()
            .cursor(CursorStyle::ResizeLeftRight)
            .flex()
            .justify_center()
            .bg(rgb(theme.bg))
            .hover(move |style| style.bg(rgb(theme.hover)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    this.begin_split_drag(f32::from(event.position.x), window, cx);
                }),
            )
            .on_click(cx.listener(|this, event: &ClickEvent, _, cx| {
                if event.click_count() >= 2 {
                    this.reset_split(cx);
                }
            }))
            .child(div().w(px(1.)).h_full().bg(rgb(theme.border)))
            .into_any_element()
    }

    pub(crate) fn render_status_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        let summary = match self.focused {
            Pane::Fs => {
                let visible = self.fs.visible();
                let files = visible
                    .iter()
                    .filter(|index| !self.fs.entries[**index].is_dir)
                    .count();
                let folders = visible
                    .iter()
                    .filter(|index| {
                        let entry = &self.fs.entries[**index];
                        entry.is_dir && !entry.is_parent
                    })
                    .count();
                format!(
                    "{} files, {} folders · {}",
                    files,
                    folders,
                    fsops::human_size(
                        visible
                            .iter()
                            .filter(|index| !self.fs.entries[**index].is_dir)
                            .map(|index| self.fs.entries[*index].size)
                            .sum()
                    )
                )
            }
            Pane::Archive => match &self.archive.listing {
                Some(listing) => format!(
                    "{} files, {} folders · {} · archive {}",
                    listing.file_count(),
                    listing.dir_count(),
                    fsops::human_size(listing.total_size()),
                    fsops::human_size(listing.archive_size)
                ),
                None => "No archive".to_string(),
            },
        };

        let selection = match self.focused {
            Pane::Fs => {
                let count = self.fs.selected.len();
                if count == 0 {
                    String::new()
                } else {
                    format!(
                        "{} selected · {}",
                        count,
                        fsops::human_size(self.fs.selected_size())
                    )
                }
            }
            Pane::Archive => {
                let count = self.archive.selected.len();
                if count == 0 {
                    String::new()
                } else {
                    format!(
                        "{} selected · {}",
                        count,
                        fsops::human_size(self.archive.selected_file_size())
                    )
                }
            }
        };

        let job_snapshot = self.job.as_ref().map(|job| {
            let job = job.lock().unwrap();
            (job.label.clone(), job.percent)
        });

        let center: AnyElement = if let Some((label, percent)) = job_snapshot {
            let width = 160.0;
            let filled = width * percent.unwrap_or(0) as f32 / 100.0;
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_color(rgb(theme.accent)).child(label))
                .child(
                    div()
                        .w(px(width))
                        .h(px(5.))
                        .rounded(px(3.))
                        .bg(rgb(theme.border))
                        .overflow_hidden()
                        .child(div().w(px(filled)).h_full().bg(rgb(theme.accent))),
                )
                .child(
                    div()
                        .w(px(36.))
                        .text_right()
                        .text_color(rgb(theme.muted))
                        .child(
                            percent
                                .map(|p| format!("{p}%"))
                                .unwrap_or_else(|| "…".to_string()),
                        ),
                )
                .child(self.btn("job-log", "Log", true, cx, |this, _, cx| {
                    let body = this.job.as_ref().map(|job| {
                        let job = job.lock().unwrap();
                        let mut body = job.label.clone();
                        body.push('\n');
                        if let Some(percent) = job.percent {
                            body.push_str(&format!("{percent}%\n"));
                        }
                        for line in &job.log {
                            body.push_str(line);
                            body.push('\n');
                        }
                        body
                    });
                    if let Some(body) = body {
                        this.modal = Some(Modal::Info {
                            title: "Operation log".into(),
                            body,
                            error: false,
                        });
                    }
                    cx.notify();
                }))
                .child(self.btn("job-cancel", "Cancel", true, cx, |this, _, cx| {
                    if let Some(job) = &this.job {
                        job::cancel(job);
                    }
                    cx.notify();
                }))
                .into_any_element()
        } else if let Some(message) = &self.message {
            div()
                .flex_1()
                .min_w(px(0.))
                .truncate()
                .text_color(rgb(if message.error {
                    theme.red
                } else {
                    theme.muted
                }))
                .child(message.text.clone())
                .into_any_element()
        } else {
            div().into_any_element()
        };

        div()
            .h(px(27.))
            .flex_none()
            .flex()
            .items_center()
            .gap_3()
            .px_3()
            .bg(rgb(theme.panel))
            .border_t_1()
            .border_color(rgb(theme.border))
            .text_xs()
            .text_color(rgb(theme.muted))
            .child(div().flex_none().child(summary))
            .child(div().flex_1().min_w(px(0.)).child(center))
            .child(
                div()
                    .flex_none()
                    .text_color(rgb(
                        if self.message.as_ref().map(|m| m.error).unwrap_or(false) {
                            theme.red
                        } else {
                            theme.muted
                        },
                    ))
                    .child(selection),
            )
            .into_any_element()
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let window_width = f32::from(window.bounds().size.width);
        let (files_width, archive_width) = self.pane_widths(window_width);
        let focus_handle = self.focus_handle.clone();
        let fs_visible = Rc::new(self.fs.visible());
        let fs_children = Rc::new(self.fs.entries.clone());
        let arc_children = Rc::new(self.archive.children());
        let arc_visible = Rc::new(self.archive.visible_indices(&arc_children));

        let mut root = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(theme.bg))
            .text_color(rgb(theme.text))
            .text_size(px(12.5))
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(Self::on_key))
            .on_mouse_move(cx.listener(Self::column_resize_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::column_resize_end))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::column_resize_end))
            .child(self.render_top_bar(window_width, cx))
            .child(self.render_tab_bar(cx))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h(px(0.))
                    .child(self.render_fs_pane(fs_visible, fs_children, files_width, cx))
                    .child(self.render_splitter(cx))
                    .child(self.render_archive_pane(arc_children, arc_visible, archive_width, cx)),
            )
            .child(self.render_status_bar(cx));

        if let Some(context) = self.render_context_menu(window, cx) {
            root = root.child(context);
        }
        if let Some(modal) = self.render_modal(cx) {
            root = root.child(modal);
        }
        root
    }
}
