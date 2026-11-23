use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, Context, SharedString, div, px, rgb};

use crate::app::AppView;
use crate::archive::NewFormat;
use crate::settings::{AudioPlayer, Setting, Settings};
use crate::theme::ThemeId;

fn default_format_id(format: NewFormat) -> &'static str {
    match format {
        NewFormat::SevenZ => "default-format-7z",
        NewFormat::Zip => "default-format-zip",
        NewFormat::Tar => "default-format-tar",
        NewFormat::TarGz => "default-format-targz",
        NewFormat::TarBz2 => "default-format-tarbz2",
        NewFormat::TarXz => "default-format-tarxz",
        NewFormat::TarZst => "default-format-tarzst",
    }
}

fn audio_player_chip_id(player: AudioPlayer) -> &'static str {
    match player {
        AudioPlayer::Ffplay => "audio-player-ffplay",
        AudioPlayer::Mpv => "audio-player-mpv",
    }
}

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
                        .flex_col()
                        .gap_2()
                        .child(div().text_sm().text_color(rgb(theme.text)).child("Preview"))
                        .child(self.settings_number_row(
                            "NFO font size",
                            "Font size used by the NFO viewer.",
                            &format!("{:.0} px", self.settings.nfo_font_size),
                            ("settings-nfo-font-dec", "settings-nfo-font-inc"),
                            cx,
                            |this, delta, cx| {
                                this.set_nfo_font_size(
                                    (this.settings.nfo_font_size + delta).clamp(10.0, 22.0),
                                    cx,
                                )
                            },
                        ))
                        .child(self.settings_number_row(
                            "General preview limit",
                            "Maximum size for non-audio files in the preview pane.",
                            &format!("{} MiB", self.settings.preview_limit_mb),
                            ("settings-preview-limit-dec", "settings-preview-limit-inc"),
                            cx,
                            |this, delta, cx| {
                                let value =
                                    (this.settings.preview_limit_mb as i64 + (delta as i64) * 4)
                                        .clamp(4, 1024) as u64;
                                this.set_preview_limit_mb(value, cx)
                            },
                        ))
                        .child(self.settings_number_row(
                            "Audio preview limit",
                            "Maximum size for audio metadata and playback.",
                            &format!("{} MiB", self.settings.audio_limit_mb),
                            ("settings-audio-limit-dec", "settings-audio-limit-inc"),
                            cx,
                            |this, delta, cx| {
                                let value =
                                    (this.settings.audio_limit_mb as i64 + (delta as i64) * 64)
                                        .clamp(16, 4096) as u64;
                                this.set_audio_limit_mb(value, cx)
                            },
                        ))
                        .child(self.settings_number_row(
                            "Text preview size",
                            "Maximum text read for code, Markdown, and NFO previews.",
                            &format!("{} KiB", self.settings.preview_text_kb),
                            ("settings-text-size-dec", "settings-text-size-inc"),
                            cx,
                            |this, delta, cx| {
                                let value =
                                    (this.settings.preview_text_kb as i64 + (delta as i64) * 64)
                                        .clamp(16, 2048) as u64;
                                this.set_preview_text_kb(value, cx)
                            },
                        ))
                        .child(self.settings_number_row(
                            "Hex preview size",
                            "Maximum bytes shown in hex previews.",
                            &format!("{} KiB", self.settings.preview_hex_kb),
                            ("settings-hex-size-dec", "settings-hex-size-inc"),
                            cx,
                            |this, delta, cx| {
                                let value =
                                    (this.settings.preview_hex_kb as i64 + (delta as i64) * 16)
                                        .clamp(4, 512) as u64;
                                this.set_preview_hex_kb(value, cx)
                            },
                        ))
                        .child(self.settings_number_row(
                            "PDF render DPI",
                            "Raster resolution used for PDF previews.",
                            &format!("{} dpi", self.settings.pdf_dpi),
                            ("settings-pdf-dpi-dec", "settings-pdf-dpi-inc"),
                            cx,
                            |this, delta, cx| {
                                let value = (this.settings.pdf_dpi as i64 + (delta as i64) * 12)
                                    .clamp(48, 300)
                                    as u32;
                                this.set_pdf_dpi(value, cx)
                            },
                        )),
                )
                .child(div().h(px(1.)).bg(rgb(theme.border)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(div().text_sm().text_color(rgb(theme.text)).child("Audio"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme.dim))
                                .child("Playback player"),
                        )
                        .child({
                            let mut players = div().flex().flex_wrap().gap_2();
                            for player in AudioPlayer::ALL {
                                players = players.child(self.chip(
                                    audio_player_chip_id(player),
                                    player.label(),
                                    self.settings.audio_player == player,
                                    cx,
                                    move |this, _, cx| this.set_audio_player(player, cx),
                                ));
                            }
                            players
                        }),
                )
                .child(div().h(px(1.)).bg(rgb(theme.border)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.text))
                                .child("Archive editing"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(theme.dim))
                                        .child("Compression level for edited archives"),
                                )
                                .child(self.compression_chips(
                                    [
                                        "edit-level-store",
                                        "edit-level-fast",
                                        "edit-level-normal",
                                        "edit-level-max",
                                    ],
                                    self.settings.edit_compression,
                                    cx,
                                    |this, level, cx| this.set_edit_compression(level, cx),
                                )),
                        ),
                )
                .child(div().h(px(1.)).bg(rgb(theme.border)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.text))
                                .child("New archive defaults"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().text_xs().text_color(rgb(theme.dim)).child("Format"))
                                .child({
                                    let mut formats = div().flex().flex_wrap().gap_2();
                                    for format in NewFormat::ALL {
                                        formats = formats.child(self.chip(
                                            default_format_id(format),
                                            format.label(),
                                            self.settings.default_format == format,
                                            cx,
                                            move |this, _, cx| this.set_default_format(format, cx),
                                        ));
                                    }
                                    formats
                                }),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(theme.dim))
                                        .child("Compression level"),
                                )
                                .child(self.compression_chips(
                                    [
                                        "default-level-store",
                                        "default-level-fast",
                                        "default-level-normal",
                                        "default-level-max",
                                    ],
                                    self.settings.default_compression,
                                    cx,
                                    |this, level, cx| this.set_default_compression(level, cx),
                                )),
                        ),
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

    fn compression_chips<F>(
        &self,
        ids: [&'static str; 4],
        current: u8,
        cx: &mut Context<Self>,
        action: F,
    ) -> AnyElement
    where
        F: Fn(&mut Self, u8, &mut Context<Self>) + Clone + 'static,
    {
        let mut chips = div().flex().flex_wrap().gap_2();
        let options = [
            (0u8, "Store", ids[0]),
            (1, "Fast", ids[1]),
            (5, "Normal", ids[2]),
            (9, "Max", ids[3]),
        ];
        for (level, label, id) in options {
            let action = action.clone();
            chips = chips.child(
                self.chip(id, label, current == level, cx, move |this, _, cx| {
                    action(this, level, cx)
                }),
            );
        }
        chips.into_any_element()
    }

    fn settings_number_row<F>(
        &self,
        title: &str,
        description: &str,
        value: &str,
        ids: (&'static str, &'static str),
        cx: &mut Context<Self>,
        action: F,
    ) -> AnyElement
    where
        F: Fn(&mut Self, f32, &mut Context<Self>) + Clone + 'static,
    {
        let theme = self.theme;
        let dec = action.clone();
        let inc = action;
        div()
            .flex()
            .items_center()
            .justify_between()
            .gap_4()
            .p_2()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme.text))
                            .child(title.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(theme.dim))
                            .child(description.to_string()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(self.btn(ids.0, "−", true, cx, move |this, _, cx| {
                        dec(this, -1.0, cx)
                    }))
                    .child(
                        div()
                            .w(px(52.))
                            .text_center()
                            .text_xs()
                            .text_color(rgb(theme.text))
                            .child(value.to_string()),
                    )
                    .child(self.btn(ids.1, "+", true, cx, move |this, _, cx| inc(this, 1.0, cx))),
            )
            .into_any_element()
    }
}
