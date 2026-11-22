use std::path::PathBuf;

use gpui::{App, Context, FocusHandle, Focusable};

use crate::columns::{ArchiveColumns, ColumnResize, FsColumns, HScrollDrag, PaneSplitDrag};
use crate::job::SharedJob;
use crate::settings::Settings;
use crate::state::{
    ArchiveState, ContextMenu, EditState, FsState, Message, Modal, Pane, PendingAction,
    PreviewSplitDrag, PreviewView, Tab,
};
use crate::theme::{Theme, ThemeId};

mod actions;
mod archive_ops;
mod context;
mod drop;
mod input;
pub(crate) mod layout;
mod modals;
mod navigation;
mod palette;
mod preferences;
mod preview;
mod search;
mod selection;
mod tabs;

pub(crate) struct AppView {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) focused: Pane,
    pub(crate) fs: FsState,
    pub(crate) archive: ArchiveState,
    pub(crate) edit: Option<EditState>,
    pub(crate) modal: Option<Modal>,
    pub(crate) context_menu: Option<ContextMenu>,
    pub(crate) preview: Option<PreviewView>,
    pub(crate) preview_generation: u64,
    pub(crate) preview_ratio: f32,
    pub(crate) preview_drag: Option<PreviewSplitDrag>,
    pub(crate) code_wrap: bool,
    pub(crate) image_zoom: f32,
    pub(crate) tabs: Vec<Tab>,
    pub(crate) active_tab: usize,
    pub(crate) job: Option<SharedJob>,
    pub(crate) last_action: Option<PendingAction>,
    pub(crate) message: Option<Message>,
    pub(crate) recent: Vec<PathBuf>,
    pub(crate) settings: Settings,
    pub(crate) fs_columns: FsColumns,
    pub(crate) archive_columns: ArchiveColumns,
    pub(crate) column_resize: Option<ColumnResize>,
    pub(crate) hscroll_drag: Option<HScrollDrag>,
    pub(crate) split_drag: Option<PaneSplitDrag>,
    pub(crate) split_ratio: f32,
    pub(crate) fs_hscroll_offset: f32,
    pub(crate) archive_hscroll_offset: f32,
    pub(crate) settings_error: Option<String>,
    pub(crate) theme_id: ThemeId,
    pub(crate) theme: Theme,
}

impl Focusable for AppView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl AppView {
    pub(crate) fn new(path: PathBuf, restore_session: bool, cx: &mut Context<Self>) -> Self {
        let settings = Settings::load();
        let fs_columns = settings.fs_columns;
        let archive_columns = settings.archive_columns;
        let theme_id = settings.theme;
        let theme = theme_id.theme();
        let split_ratio = settings.split;
        let mut fs = FsState::new(path);
        fs.show_hidden = settings.show_hidden;
        fs.clamp_active();
        eprintln!("startup: AppView::new begin");
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            focused: Pane::Fs,
            fs,
            archive: ArchiveState::new(),
            edit: None,
            modal: None,
            context_menu: None,
            preview: None,
            preview_generation: 0,
            preview_ratio: settings.preview_ratio,
            preview_drag: None,
            code_wrap: false,
            image_zoom: 1.0,
            tabs: vec![Tab::empty()],
            active_tab: 0,
            job: None,
            last_action: None,
            message: None,
            recent: Vec::new(),
            settings,
            fs_columns,
            archive_columns,
            column_resize: None,
            hscroll_drag: None,
            split_drag: None,
            split_ratio,
            fs_hscroll_offset: 0.0,
            archive_hscroll_offset: 0.0,
            settings_error: None,
            theme_id,
            theme,
        };
        eprintln!("startup: before restore");
        view.restore_session(restore_session, cx);
        eprintln!("startup: AppView::new end");
        view
    }

    pub(crate) fn set_message(&mut self, text: impl Into<String>, error: bool) {
        self.message = Some(Message {
            text: text.into(),
            error,
        });
    }

    pub(crate) fn push_recent(&mut self, path: PathBuf) {
        self.recent.retain(|item| item != &path);
        self.recent.insert(0, path);
        self.recent.truncate(10);
    }
}
