use gpui::{Context, MouseMoveEvent, MouseUpEvent, Window};

use crate::columns::{
    ArchiveColumns, ColumnId, ColumnResize, FsColumns, HScrollDrag, ICON_WIDTH, PaneSplitDrag,
    max_width, min_width,
};
use crate::state::Pane;

use super::AppView;

pub(crate) const SPLITTER_WIDTH: f32 = 6.0;
pub(crate) const MIN_PANE_WIDTH: f32 = 280.0;

impl AppView {
    pub(crate) fn begin_column_resize(
        &mut self,
        pane: Pane,
        column: ColumnId,
        start_x: f32,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let content = self.pane_content_width(window, pane);
        let start_width = match pane {
            Pane::Fs => {
                if self.fs_columns.name.is_none() {
                    self.fs_columns.name = Some(
                        (content - self.fs_columns.fixed_total()).max(min_width(ColumnId::Name)),
                    );
                }
                match column {
                    ColumnId::Name => self.fs_columns.name.unwrap_or(min_width(ColumnId::Name)),
                    ColumnId::Size => self.fs_columns.size,
                    ColumnId::Modified => self.fs_columns.modified,
                    ColumnId::Kind => self.fs_columns.kind,
                    ColumnId::Packed => return,
                }
            }
            Pane::Archive => {
                if self.archive_columns.name.is_none() {
                    self.archive_columns.name = Some(
                        (content - self.archive_columns.fixed_total())
                            .max(min_width(ColumnId::Name)),
                    );
                }
                match column {
                    ColumnId::Name => self
                        .archive_columns
                        .name
                        .unwrap_or(min_width(ColumnId::Name)),
                    ColumnId::Size => self.archive_columns.size,
                    ColumnId::Packed => self.archive_columns.packed,
                    ColumnId::Modified => self.archive_columns.modified,
                    ColumnId::Kind => self.archive_columns.method,
                }
            }
        };
        self.column_resize = Some(ColumnResize {
            pane,
            column,
            start_x,
            start_width,
        });
        cx.notify();
    }

    pub(crate) fn resize_column(&mut self, x: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.column_resize else {
            return;
        };
        let width = (drag.start_width + x - drag.start_x)
            .clamp(min_width(drag.column), max_width(drag.column));
        self.set_column(drag.pane, drag.column, width);
        cx.notify();
    }

    pub(crate) fn pane_widths(&self, window_width: f32) -> (f32, f32) {
        let available = (window_width - SPLITTER_WIDTH).max(1.0);
        let min_ratio = (MIN_PANE_WIDTH / available).clamp(0.05, 0.5);
        let ratio = self.split_ratio.clamp(min_ratio, 1.0 - min_ratio);
        let files = (available * ratio).floor();
        (files, (available - files).max(0.0))
    }

    pub(crate) fn begin_split_drag(&mut self, x: f32, window: &Window, cx: &mut Context<Self>) {
        let content_width = (f32::from(window.bounds().size.width) - SPLITTER_WIDTH).max(1.0);
        self.split_drag = Some(PaneSplitDrag {
            start_x: x,
            start_ratio: self.split_ratio,
            content_width,
        });
        cx.notify();
    }

    fn split_move(&mut self, x: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.split_drag else {
            return;
        };
        let min_ratio = (MIN_PANE_WIDTH / drag.content_width).clamp(0.05, 0.5);
        let ratio = (drag.start_ratio + (x - drag.start_x) / drag.content_width)
            .clamp(min_ratio, 1.0 - min_ratio);
        self.split_ratio = ratio;
        cx.notify();
    }

    fn save_split(&mut self) {
        self.settings.split = self.split_ratio;
        match self.settings.save() {
            Ok(()) => self.settings_error = None,
            Err(error) => self.settings_error = Some(error),
        }
    }

    pub(crate) fn reset_split(&mut self, cx: &mut Context<Self>) {
        self.split_ratio = 0.5;
        self.split_drag = None;
        self.save_split();
        cx.notify();
    }

    pub(crate) fn begin_preview_drag(&mut self, y: f32, window: &Window, cx: &mut Context<Self>) {
        self.preview_drag = Some(crate::state::PreviewSplitDrag {
            start_y: y,
            start_ratio: self.preview_ratio,
            height: f32::from(window.bounds().size.height).max(1.0),
        });
        cx.notify();
    }

    fn preview_move(&mut self, y: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.preview_drag else {
            return;
        };
        self.preview_ratio =
            (drag.start_ratio + (drag.start_y - y) / drag.height).clamp(0.15, 0.75);
        cx.notify();
    }

    fn save_preview_ratio(&mut self) {
        self.settings.preview_ratio = self.preview_ratio;
        if let Err(error) = self.settings.save() {
            self.settings_error = Some(error);
        }
    }

    pub(crate) fn reset_preview_ratio(&mut self, cx: &mut Context<Self>) {
        self.preview_ratio = 0.32;
        self.preview_drag = None;
        self.save_preview_ratio();
        cx.notify();
    }

    pub(crate) fn begin_hscroll_drag(&mut self, drag: HScrollDrag, cx: &mut Context<Self>) {
        self.hscroll_drag = Some(drag);
        cx.notify();
    }

    fn hscroll_move(&mut self, x: f32, cx: &mut Context<Self>) {
        let Some(drag) = self.hscroll_drag else {
            return;
        };
        let travel = (drag.track_width - drag.thumb_width).max(1.0);
        let offset = (drag.start_offset + (x - drag.start_x) / travel * drag.max_scroll)
            .clamp(0.0, drag.max_scroll);
        match drag.pane {
            Pane::Fs => self.fs_hscroll_offset = offset,
            Pane::Archive => self.archive_hscroll_offset = offset,
        }
        cx.notify();
    }

    pub(crate) fn column_resize_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.preview_drag.is_some() {
            self.preview_move(f32::from(event.position.y), cx);
            return;
        }
        if self.split_drag.is_some() {
            self.split_move(f32::from(event.position.x), cx);
            return;
        }
        if self.hscroll_drag.is_some() {
            self.hscroll_move(f32::from(event.position.x), cx);
            return;
        }
        if self.column_resize.is_some() {
            self.resize_column(f32::from(event.position.x), cx);
        }
    }

    pub(crate) fn column_resize_end(
        &mut self,
        _: &MouseUpEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.preview_drag.take().is_some() {
            self.save_preview_ratio();
            cx.notify();
            return;
        }
        if self.split_drag.take().is_some() {
            self.save_split();
            cx.notify();
            return;
        }
        if self.hscroll_drag.take().is_some() {
            cx.notify();
            return;
        }
        if self.column_resize.take().is_some() {
            self.save_columns();
            cx.notify();
        }
    }

    pub(crate) fn reset_columns(&mut self, cx: &mut Context<Self>) {
        self.fs_columns = FsColumns::default();
        self.archive_columns = ArchiveColumns::default();
        self.fs_hscroll_offset = 0.0;
        self.archive_hscroll_offset = 0.0;
        self.column_resize = None;
        self.save_columns();
        cx.notify();
    }

    pub(crate) fn fs_inner_width(&self, pane_width: f32) -> f32 {
        self.fs_columns.table_width(32.0).map_or_else(
            || pane_width.max(self.fs_columns.min_table_width(32.0)),
            |width| width.max(pane_width),
        )
    }

    pub(crate) fn archive_inner_width(&self, pane_width: f32) -> f32 {
        self.archive_columns.table_width(40.0).map_or_else(
            || pane_width.max(self.archive_columns.min_table_width(40.0)),
            |width| width.max(pane_width),
        )
    }

    fn save_columns(&mut self) {
        self.settings.fs_columns = self.fs_columns;
        self.settings.archive_columns = self.archive_columns;
        match self.settings.save() {
            Ok(()) => self.settings_error = None,
            Err(error) => self.settings_error = Some(error),
        }
    }

    fn set_column(&mut self, pane: Pane, column: ColumnId, width: f32) {
        match pane {
            Pane::Fs => self.fs_columns.set(column, width),
            Pane::Archive => self.archive_columns.set(column, width),
        }
    }
}

impl AppView {
    fn pane_content_width(&self, window: &Window, pane: Pane) -> f32 {
        let (files, archive) = self.pane_widths(f32::from(window.bounds().size.width));
        let pane_width = match pane {
            Pane::Fs => files,
            Pane::Archive => archive,
        };
        let gaps = match pane {
            Pane::Fs => 32.0,
            Pane::Archive => 40.0,
        };
        pane_width - 16.0 - ICON_WIDTH - gaps
    }
}
