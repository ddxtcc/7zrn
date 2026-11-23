use gpui::Context;

use crate::settings::{AudioPlayer, Setting};
use crate::state::Modal;
use crate::theme::ThemeId;

use super::AppView;

impl AppView {
    pub(crate) fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.modal = Some(Modal::Settings);
        self.edit = None;
        cx.notify();
    }

    pub(crate) fn setting_value(&self, setting: Setting) -> bool {
        setting.value(&self.settings)
    }

    pub(crate) fn set_setting(&mut self, setting: Setting, value: bool, cx: &mut Context<Self>) {
        setting.set(&mut self.settings, value);
        if setting == Setting::ShowHidden {
            self.fs.show_hidden = value;
            self.fs.clamp_active();
        }
        if setting == Setting::CodeWrap {
            self.code_wrap = value;
        }
        match self.settings.save() {
            Ok(()) => self.settings_error = None,
            Err(error) => self.settings_error = Some(error),
        }
        cx.notify();
    }

    pub(crate) fn toggle_setting(&mut self, setting: Setting, cx: &mut Context<Self>) {
        let value = !self.setting_value(setting);
        self.set_setting(setting, value, cx);
    }

    pub(crate) fn set_nfo_font_size(&mut self, size: f32, cx: &mut Context<Self>) {
        self.settings.nfo_font_size = size.clamp(10.0, 22.0);
        self.save_settings(cx);
    }

    pub(crate) fn set_edit_compression(&mut self, value: u8, cx: &mut Context<Self>) {
        self.settings.edit_compression = value.min(9);
        self.save_settings(cx);
    }

    pub(crate) fn set_default_compression(&mut self, value: u8, cx: &mut Context<Self>) {
        self.settings.default_compression = value.min(9);
        self.save_settings(cx);
    }

    pub(crate) fn set_preview_limit_mb(&mut self, value: u64, cx: &mut Context<Self>) {
        self.settings.preview_limit_mb = value.clamp(4, 1024);
        self.save_settings(cx);
    }

    pub(crate) fn set_audio_limit_mb(&mut self, value: u64, cx: &mut Context<Self>) {
        self.settings.audio_limit_mb = value.clamp(16, 4096);
        self.save_settings(cx);
    }

    pub(crate) fn set_preview_text_kb(&mut self, value: u64, cx: &mut Context<Self>) {
        self.settings.preview_text_kb = value.clamp(16, 2048);
        self.save_settings(cx);
    }

    pub(crate) fn set_preview_hex_kb(&mut self, value: u64, cx: &mut Context<Self>) {
        self.settings.preview_hex_kb = value.clamp(4, 512);
        self.save_settings(cx);
    }

    pub(crate) fn set_pdf_dpi(&mut self, value: u32, cx: &mut Context<Self>) {
        self.settings.pdf_dpi = value.clamp(48, 300);
        self.save_settings(cx);
    }

    pub(crate) fn set_audio_player(&mut self, player: AudioPlayer, cx: &mut Context<Self>) {
        self.settings.audio_player = player;
        self.save_settings(cx);
    }

    pub(crate) fn set_default_format(
        &mut self,
        format: crate::archive::NewFormat,
        cx: &mut Context<Self>,
    ) {
        self.settings.default_format = format;
        self.save_settings(cx);
    }

    fn save_settings(&mut self, cx: &mut Context<Self>) {
        match self.settings.save() {
            Ok(()) => self.settings_error = None,
            Err(error) => self.settings_error = Some(error),
        }
        cx.notify();
    }

    pub(crate) fn set_theme(&mut self, theme_id: ThemeId, cx: &mut Context<Self>) {
        self.theme_id = theme_id;
        self.theme = theme_id.theme();
        self.settings.theme = theme_id;
        match self.settings.save() {
            Ok(()) => self.settings_error = None,
            Err(error) => self.settings_error = Some(error),
        }
        cx.notify();
    }
}
