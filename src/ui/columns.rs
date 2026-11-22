use gpui::prelude::*;
use gpui::{
    AnyElement, Context, CursorStyle, MouseButton, MouseDownEvent, SharedString, div, px, rgb,
};

use crate::app::AppView;
use crate::columns::{ColumnId, ICON_WIDTH, min_width};
use crate::state::Pane;

#[derive(Clone, Copy)]
struct HeaderCell {
    label: &'static str,
    width: Option<f32>,
    id: &'static str,
    pane: Pane,
    column: Option<ColumnId>,
}

impl AppView {
    fn header_cell(&self, cell: HeaderCell, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        let mut header = div().relative().h_full().flex().items_center();
        header = match cell.width {
            Some(width) => header.w(px(width)).flex_none(),
            None => header.flex_1().min_w(px(min_width(ColumnId::Name))),
        };
        header = header.child(cell.label).child(
            div()
                .absolute()
                .top(px(4.))
                .bottom(px(4.))
                .right_0()
                .w(px(1.))
                .bg(rgb(theme.border)),
        );
        if let Some(column) = cell.column {
            header = header.child(
                div()
                    .id(SharedString::from(format!("resize-{}", cell.id)))
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right(px(-2.))
                    .w(px(6.))
                    .cursor(CursorStyle::ResizeLeftRight)
                    .hover(move |style| style.bg(rgb(theme.border2)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                            this.begin_column_resize(
                                cell.pane,
                                column,
                                f32::from(event.position.x),
                                window,
                                cx,
                            );
                        }),
                    ),
            );
        }
        header.into_any_element()
    }

    pub(crate) fn render_fs_columns(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        div()
            .h(px(24.))
            .w_full()
            .flex_none()
            .flex()
            .items_center()
            .px_2()
            .gap_2()
            .bg(rgb(theme.panel))
            .border_b_1()
            .border_color(rgb(theme.border))
            .text_xs()
            .text_color(rgb(theme.dim))
            .child(div().w(px(ICON_WIDTH)).flex_none())
            .child(self.header_cell(
                HeaderCell {
                    label: "Name",
                    width: self.fs_columns.name,
                    id: "fs-name",
                    pane: Pane::Fs,
                    column: Some(ColumnId::Name),
                },
                cx,
            ))
            .child(self.header_cell(
                HeaderCell {
                    label: "Size",
                    width: Some(self.fs_columns.size),
                    id: "fs-size",
                    pane: Pane::Fs,
                    column: Some(ColumnId::Size),
                },
                cx,
            ))
            .child(self.header_cell(
                HeaderCell {
                    label: "Modified",
                    width: Some(self.fs_columns.modified),
                    id: "fs-modified",
                    pane: Pane::Fs,
                    column: Some(ColumnId::Modified),
                },
                cx,
            ))
            .child(self.header_cell(
                HeaderCell {
                    label: "Type",
                    width: Some(self.fs_columns.kind),
                    id: "fs-kind",
                    pane: Pane::Fs,
                    column: Some(ColumnId::Kind),
                },
                cx,
            ))
            .into_any_element()
    }

    pub(crate) fn render_archive_columns(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        div()
            .h(px(24.))
            .w_full()
            .flex_none()
            .flex()
            .items_center()
            .px_2()
            .gap_2()
            .bg(rgb(theme.panel))
            .border_b_1()
            .border_color(rgb(theme.border))
            .text_xs()
            .text_color(rgb(theme.dim))
            .child(div().w(px(ICON_WIDTH)).flex_none())
            .child(self.header_cell(
                HeaderCell {
                    label: "Name",
                    width: self.archive_columns.name,
                    id: "archive-name",
                    pane: Pane::Archive,
                    column: Some(ColumnId::Name),
                },
                cx,
            ))
            .child(self.header_cell(
                HeaderCell {
                    label: "Size",
                    width: Some(self.archive_columns.size),
                    id: "archive-size",
                    pane: Pane::Archive,
                    column: Some(ColumnId::Size),
                },
                cx,
            ))
            .child(self.header_cell(
                HeaderCell {
                    label: "Packed",
                    width: Some(self.archive_columns.packed),
                    id: "archive-packed",
                    pane: Pane::Archive,
                    column: Some(ColumnId::Packed),
                },
                cx,
            ))
            .child(self.header_cell(
                HeaderCell {
                    label: "Modified",
                    width: Some(self.archive_columns.modified),
                    id: "archive-modified",
                    pane: Pane::Archive,
                    column: Some(ColumnId::Modified),
                },
                cx,
            ))
            .child(self.header_cell(
                HeaderCell {
                    label: "Method",
                    width: Some(self.archive_columns.method),
                    id: "archive-method",
                    pane: Pane::Archive,
                    column: Some(ColumnId::Kind),
                },
                cx,
            ))
            .into_any_element()
    }
}
