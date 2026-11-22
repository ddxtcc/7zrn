use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, Context, SharedString, div, px, rgb};

use crate::app::AppView;
use crate::settings::{Setting, Settings};
use crate::theme::ThemeId;

fn theme_chip_id(theme: ThemeId) -> &'static str {
    match theme {
        ThemeId::Dark => "theme-dark",
        ThemeId::Light => "theme-light",
        ThemeId::Nord => "theme-nord",
        ThemeId::Gruvbox => "theme-gruvbox",
        ThemeId::Dracula => "theme-dracula",
        ThemeId::Solarized => "theme-solarized",
    }
}

impl AppView {
    pub(crate) fn settings_toggle_row(
        &self,
        setting: Setting,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = self.theme;
        let value = self.setting_value(setting);
        div()
            .id(SharedString::from(format!("setting-{}", setting.label())))
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .p_2()
            .rounded(px(4.))
            .cursor_pointer()
            .hover(move |style| style.bg(rgb(theme.hover)))
            .on_click(
                cx.listener(move |this, _: &ClickEvent, _, cx| this.toggle_setting(setting, cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme.text))
                            .child(setting.label()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(theme.dim))
                            .child(setting.description()),
                    ),
            )
            .child(
                div()
                    .w(px(36.))
                    .h(px(20.))
                    .flex_none()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(rgb(if value { theme.accent } else { theme.border }))
                    .bg(rgb(if value {
                        theme.accent_dim
                    } else {
                        theme.panel2
                    }))
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .size(px(12.))
                            .rounded(px(6.))
                            .ml(px(if value { 20. } else { 3. }))
                            .bg(rgb(if value { theme.accent } else { theme.muted })),
                    ),
            )
            .into_any_element()
    }

    pub(crate) fn render_settings_modal(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;

        let mut rows = div().flex().flex_col().gap_2();
        for setting in Setting::ALL {
            rows = rows.child(self.settings_toggle_row(setting, cx));
        }
        let mut themes = div().flex().flex_wrap().gap_2();
        for theme_id in ThemeId::ALL {
            themes = themes.child(self.chip(
                theme_chip_id(theme_id),
                theme_id.label(),
                self.theme_id == theme_id,
                cx,
                move |this, _, cx| this.set_theme(theme_id, cx),
            ));
        }
        self.modal_panel_scroll(
            "Settings",
            640.0,
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(rows)
                .child(div().h(px(1.)).bg(rgb(theme.border)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().text_sm().text_color(rgb(theme.text)).child("Theme"))
                        .child(themes),
                )
                .child(div().h(px(1.)).bg(rgb(theme.border)))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_4()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(theme.text))
                                        .child("Column widths"),
                                )
                                .child(
                                    div().text_xs().text_color(rgb(theme.dim)).child(
                                        "Drag the right edge of a column header to resize it.",
                                    ),
                                ),
                        )
                        .child(self.btn(
                            "settings-reset-columns",
                            "Reset",
                            true,
                            cx,
                            |this, _, cx| this.reset_columns(cx),
                        )),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_4()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(theme.text))
                                        .child("Panel split"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(theme.dim))
                                        .child("Drag the divider between Files and Archive."),
                                ),
                        )
                        .child(self.btn(
                            "settings-reset-split",
                            "50/50",
                            true,
                            cx,
                            |this, _, cx| this.reset_split(cx),
                        )),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.dim))
                        .truncate()
                        .child(format!("Settings file: {}", Settings::path().display())),
                )
                .when_some(self.settings_error.clone(), |this, error| {
                    this.child(div().text_xs().text_color(rgb(theme.red)).child(error))
                }),
            div().flex().items_center().gap_2().child(self.primary_btn(
                "settings-close",
                "Close",
                false,
                cx,
                |this, _, cx| this.close_modal(cx),
            )),
            cx,
        )
    }
}
