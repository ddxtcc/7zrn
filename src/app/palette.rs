use gpui::{Context, Window};

use crate::palette::{self, PaletteAction};
use crate::settings::Setting;
use crate::state::{EditTarget, Modal};

use super::AppView;

impl AppView {
    pub(crate) fn open_palette_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.modal = Some(Modal::Palette {
            query: String::new(),
            selected: 0,
        });
        self.begin_edit(EditTarget::Palette, String::new(), window, cx);
    }

    pub(crate) fn update_palette_selection(&mut self) {
        if let Some(Modal::Palette { query, selected }) = &mut self.modal {
            let count = palette::matches(query).len();
            if count == 0 {
                *selected = 0;
            } else if *selected >= count {
                *selected = count - 1;
            }
        }
    }

    pub(crate) fn move_palette_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        let query = match &self.modal {
            Some(Modal::Palette { query, .. }) => query.clone(),
            _ => return,
        };
        let count = palette::matches(&query).len() as isize;
        if count == 0 {
            return;
        }
        if let Some(Modal::Palette { selected, .. }) = &mut self.modal {
            *selected = (*selected as isize + delta).clamp(0, count - 1) as usize;
        }
        cx.notify();
    }

    pub(crate) fn activate_palette_action(
        &mut self,
        action: PaletteAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.modal = None;
        self.edit = None;
        match action {
            PaletteAction::NewTab => self.new_tab(cx),
            PaletteAction::CloseTab => self.close_tab(self.active_tab, cx),
            PaletteAction::NextTab => self.cycle_tab(true, cx),
            PaletteAction::PreviousTab => self.cycle_tab(false, cx),
            PaletteAction::Search => self.open_search_modal(window, cx),
            PaletteAction::Properties => self.open_archive_properties(cx),
            PaletteAction::Extract => self.open_extract_modal(window, cx),
            PaletteAction::Add => self.add_to_archive(cx),
            PaletteAction::Delete => self.request_delete(cx),
            PaletteAction::Rename => self.open_rename_modal(window, cx),
            PaletteAction::NewFolder => self.open_new_folder_modal(window, cx),
            PaletteAction::Test => self.test_archive(cx),
            PaletteAction::Refresh => match self.focused {
                crate::state::Pane::Fs => self.refresh_fs(cx),
                crate::state::Pane::Archive => self.refresh_archive(cx),
            },
            PaletteAction::Settings => self.open_settings(cx),
            PaletteAction::Help => {
                self.modal = Some(Modal::Help);
            }
            PaletteAction::ToggleHidden => self.toggle_setting(Setting::ShowHidden, cx),
        }
        cx.notify();
    }

    pub(crate) fn submit_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let query = match &self.modal {
            Some(Modal::Palette { query, .. }) => query.clone(),
            _ => return,
        };
        let selected = match &self.modal {
            Some(Modal::Palette { selected, .. }) => *selected,
            _ => 0,
        };
        if let Some(action) = palette::matches(&query).get(selected).copied() {
            self.activate_palette_action(action, window, cx);
        }
    }
}
