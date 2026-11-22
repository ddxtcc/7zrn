use crate::state::Pane;

pub(crate) const ICON_WIDTH: f32 = 7.0;
const MAX_WIDTH: f32 = 420.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ColumnId {
    Name,
    Size,
    Packed,
    Modified,
    Kind,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PaneSplitDrag {
    pub(crate) start_x: f32,
    pub(crate) start_ratio: f32,
    pub(crate) content_width: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct HScrollDrag {
    pub(crate) pane: Pane,
    pub(crate) start_x: f32,
    pub(crate) start_offset: f32,
    pub(crate) max_scroll: f32,
    pub(crate) track_width: f32,
    pub(crate) thumb_width: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ColumnResize {
    pub(crate) pane: Pane,
    pub(crate) column: ColumnId,
    pub(crate) start_x: f32,
    pub(crate) start_width: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FsColumns {
    pub(crate) name: Option<f32>,
    pub(crate) size: f32,
    pub(crate) modified: f32,
    pub(crate) kind: f32,
}

impl Default for FsColumns {
    fn default() -> Self {
        Self {
            name: None,
            size: 76.0,
            modified: 126.0,
            kind: 74.0,
        }
    }
}

impl FsColumns {
    pub(crate) fn set(&mut self, column: ColumnId, width: f32) {
        match column {
            ColumnId::Name => self.name = Some(width),
            ColumnId::Size => self.size = width,
            ColumnId::Modified => self.modified = width,
            ColumnId::Kind => self.kind = width,
            ColumnId::Packed => {}
        }
    }

    pub(crate) fn fixed_total(self) -> f32 {
        self.size + self.modified + self.kind
    }

    pub(crate) fn table_width(self, gaps: f32) -> Option<f32> {
        self.name
            .map(|name| 16.0 + ICON_WIDTH + gaps + name + self.fixed_total())
    }

    pub(crate) fn min_table_width(self, gaps: f32) -> f32 {
        16.0 + ICON_WIDTH + gaps + min_width(ColumnId::Name) + self.fixed_total()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ArchiveColumns {
    pub(crate) name: Option<f32>,
    pub(crate) size: f32,
    pub(crate) packed: f32,
    pub(crate) modified: f32,
    pub(crate) method: f32,
}

impl Default for ArchiveColumns {
    fn default() -> Self {
        Self {
            name: None,
            size: 72.0,
            packed: 68.0,
            modified: 120.0,
            method: 82.0,
        }
    }
}

impl ArchiveColumns {
    pub(crate) fn set(&mut self, column: ColumnId, width: f32) {
        match column {
            ColumnId::Name => self.name = Some(width),
            ColumnId::Size => self.size = width,
            ColumnId::Packed => self.packed = width,
            ColumnId::Modified => self.modified = width,
            ColumnId::Kind => self.method = width,
        }
    }

    pub(crate) fn fixed_total(self) -> f32 {
        self.size + self.packed + self.modified + self.method
    }

    pub(crate) fn table_width(self, gaps: f32) -> Option<f32> {
        self.name
            .map(|name| 16.0 + ICON_WIDTH + gaps + name + self.fixed_total())
    }

    pub(crate) fn min_table_width(self, gaps: f32) -> f32 {
        16.0 + ICON_WIDTH + gaps + min_width(ColumnId::Name) + self.fixed_total()
    }
}

pub(crate) fn min_width(column: ColumnId) -> f32 {
    match column {
        ColumnId::Name => 60.0,
        ColumnId::Size => 56.0,
        ColumnId::Packed => 50.0,
        ColumnId::Modified => 90.0,
        ColumnId::Kind => 56.0,
    }
}

pub(crate) fn max_width(column: ColumnId) -> f32 {
    match column {
        ColumnId::Name => 1200.0,
        _ => MAX_WIDTH,
    }
}
