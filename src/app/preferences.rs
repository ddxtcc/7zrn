use gpui::Context;

use crate::settings::Setting;
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
