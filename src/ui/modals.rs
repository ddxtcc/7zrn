use gpui::prelude::*;
use gpui::{
    AnyElement, Context, IntoElement, MouseButton, MouseDownEvent, deferred, div, px, rgb, rgba,
};

use crate::app::AppView;
use crate::archive::NewFormat;
use crate::settings::Setting;
use crate::state::{EditTarget, Modal};
use crate::ui::widgets::*;

fn when_selected_only_hint(selected_only: bool) -> &'static str {
    if selected_only {
        "Selected entries keep their path inside the destination."
    } else {
        "Extracts every entry in the archive."
    }
}

fn format_chip_id(format: NewFormat) -> &'static str {
    match format {
        NewFormat::SevenZ => "new-format-7z",
        NewFormat::Zip => "new-format-zip",
        NewFormat::Tar => "new-format-tar",
        NewFormat::TarGz => "new-format-targz",
        NewFormat::TarBz2 => "new-format-tarbz2",
        NewFormat::TarXz => "new-format-tarxz",
        NewFormat::TarZst => "new-format-tarzst",
    }
}

impl AppView {
    pub(crate) fn render_modal(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let theme = self.theme;
        let modal = self.modal.as_ref()?;
        let panel: AnyElement = match modal {
            Modal::NewArchive {
                format,
                names,
                base: _,
                name,
                error,
            } => {
                let name_text = match &self.edit {
                    Some(edit) if edit.target == EditTarget::NewName => edit.text.clone(),
                    _ => name.clone(),
                };
                let active = self
                    .edit
                    .as_ref()
                    .map(|edit| edit.target == EditTarget::NewName)
                    .unwrap_or(false);
                let cursor = self
                    .edit
                    .as_ref()
                    .filter(|edit| edit.target == EditTarget::NewName)
                    .map(|edit| edit.cursor)
                    .unwrap_or(0);
                let selected_format = *format;
                let mut formats = div().flex().flex_wrap().gap_2();
                for option in NewFormat::ALL {
                    formats = formats.child(self.chip(
                        format_chip_id(option),
                        option.label(),
                        option == selected_format,
                        cx,
                        move |this, _, cx| this.set_new_format(option, cx),
                    ));
                }
                self.modal_panel(
                    "New archive",
                    560.0,
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div().text_sm().text_color(rgb(theme.muted)).child(format!(
                                "Archive {} item(s) from this folder.",
                                names.len()
                            )),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme.dim))
                                .child("Archive path"),
                        )
                        .child(self.input_field(
                            InputField {
                                id: "new-name",
                                text: &name_text,
                                placeholder: "Archive path",
                                active,
                                cursor,
                                width: None,
                                target: Some(EditTarget::NewName),
                                click_text: name_text.clone(),
                            },
                            cx,
                        ))
                        .child(formats)
                        .when_some(error.clone(), |this, error| {
                            this.child(div().text_xs().text_color(rgb(theme.red)).child(error))
                        }),
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(self.btn("new-cancel", "Cancel", true, cx, |this, _, cx| {
                            this.close_modal(cx)
                        }))
                        .child(self.primary_btn(
                            "new-create",
                            "Create",
                            false,
                            cx,
                            |this, _, cx| this.submit_modal(cx),
                        )),
                    cx,
                )
            }
            Modal::Extract {
                dest,
                selected_only,
                count_selected,
                count_total,
                error,
            } => {
                let dest_text = match &self.edit {
                    Some(edit) if edit.target == EditTarget::ExtractDest => edit.text.clone(),
                    _ => dest.clone(),
                };
                let active = self
                    .edit
                    .as_ref()
                    .map(|edit| edit.target == EditTarget::ExtractDest)
                    .unwrap_or(false);
                let cursor = self
                    .edit
                    .as_ref()
                    .filter(|edit| edit.target == EditTarget::ExtractDest)
                    .map(|edit| edit.cursor)
                    .unwrap_or(0);
                let count = if *selected_only {
                    *count_selected
                } else {
                    *count_total
                };
                let selected = *selected_only;
                self.modal_panel(
                    "Extract archive",
                    560.0,
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.muted))
                                .child(format!("Extract {count} item(s) to:")),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(self.input_field(
                                    InputField {
                                        id: "extract-dest",
                                        text: &dest_text,
                                        placeholder: "Destination",
                                        active,
                                        cursor,
                                        width: None,
                                        target: Some(EditTarget::ExtractDest),
                                        click_text: dest_text.clone(),
                                    },
                                    cx,
                                ))
                                .child(self.btn(
                                    "extract-browse",
                                    "Browse",
                                    true,
                                    cx,
                                    |this, window, cx| this.browse_extract_destination(window, cx),
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(self.chip(
                                    "extract-selected",
                                    &format!("Selected ({count_selected})"),
                                    selected,
                                    cx,
                                    |this, _, cx| this.set_extract_scope(true, cx),
                                ))
                                .child(self.chip(
                                    "extract-all",
                                    &format!("All ({count_total})"),
                                    !selected,
                                    cx,
                                    |this, _, cx| this.set_extract_scope(false, cx),
                                )),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme.dim))
                                .child(when_selected_only_hint(selected)),
                        )
                        .child(self.settings_toggle_row(Setting::OpenAfterExtract, cx))
                        .when_some(error.clone(), |this, error| {
                            this.child(div().text_xs().text_color(rgb(theme.red)).child(error))
                        }),
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            self.btn("extract-cancel", "Cancel", true, cx, |this, _, cx| {
                                this.close_modal(cx)
                            }),
                        )
                        .child(self.primary_btn(
                            "extract-go",
                            "Extract",
                            false,
                            cx,
                            |this, _, cx| this.submit_modal(cx),
                        )),
                    cx,
                )
            }
            Modal::DeleteConfirm { display_names, .. } => {
                let mut lines = div().flex().flex_col().gap_1();
                for name in display_names.iter().take(8) {
                    lines = lines.child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme.text))
                            .truncate()
                            .child(name.clone()),
                    );
                }
                if display_names.len() > 8 {
                    lines = lines.child(
                        div()
                            .text_xs()
                            .text_color(rgb(theme.dim))
                            .child(format!("… and {} more", display_names.len() - 8)),
                    );
                }
                self.modal_panel(
                    "Delete entries?",
                    520.0,
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(div().text_sm().text_color(rgb(theme.muted)).child(format!(
                            "Remove {} item(s) from this archive?",
                            display_names.len()
                        )))
                        .child(lines)
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme.amber))
                                .child("This cannot be undone."),
                        ),
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            self.btn("delete-cancel", "Cancel", true, cx, |this, _, cx| {
                                this.close_modal(cx)
                            }),
                        )
                        .child(self.primary_btn(
                            "delete-confirm",
                            "Delete",
                            true,
                            cx,
                            |this, _, cx| this.submit_modal(cx),
                        )),
                    cx,
                )
            }
            Modal::Info { title, body, error } => {
                let mut lines = div().flex().flex_col().gap_1();
                for line in body.lines() {
                    lines = lines.child(
                        div()
                            .text_sm()
                            .text_color(rgb(if *error { theme.red } else { theme.muted }))
                            .child(if line.is_empty() {
                                " ".to_string()
                            } else {
                                line.to_string()
                            }),
                    );
                }
                self.modal_panel(
                    title,
                    680.0,
                    div()
                        .id("info-body")
                        .max_h(gpui::relative(0.55))
                        .min_h(px(0.))
                        .overflow_y_scroll()
                        .child(lines),
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(self.btn("info-copy", "Copy", true, cx, |this, _, cx| {
                            this.copy_modal_body(cx)
                        }))
                        .child(self.primary_btn(
                            "info-close",
                            "Close",
                            false,
                            cx,
                            |this, _, cx| this.close_modal(cx),
                        )),
                    cx,
                )
            }
            Modal::Password {
                message,
                password,
                error,
                ..
            } => {
                let password_text = match &self.edit {
                    Some(edit) if edit.target == EditTarget::Password => edit.text.clone(),
                    _ => password.clone(),
                };
                let active = self
                    .edit
                    .as_ref()
                    .map(|edit| edit.target == EditTarget::Password)
                    .unwrap_or(false);
                let cursor = self
                    .edit
                    .as_ref()
                    .filter(|edit| edit.target == EditTarget::Password)
                    .map(|edit| edit.cursor)
                    .unwrap_or(0);
                self.modal_panel(
                    "Password required",
                    560.0,
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.muted))
                                .child(message.clone()),
                        )
                        .child(self.input_field(
                            InputField {
                                id: "password-input",
                                text: &password_text,
                                placeholder: "Password",
                                active,
                                cursor,
                                width: None,
                                target: Some(EditTarget::Password),
                                click_text: password_text.clone(),
                            },
                            cx,
                        ))
                        .when_some(error.clone(), |this, error| {
                            this.child(div().text_xs().text_color(rgb(theme.red)).child(error))
                        }),
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            self.btn("password-cancel", "Cancel", true, cx, |this, _, cx| {
                                this.close_modal(cx)
                            }),
                        )
                        .child(self.primary_btn(
                            "password-submit",
                            "Unlock",
                            false,
                            cx,
                            |this, _, cx| this.submit_modal(cx),
                        )),
                    cx,
                )
            }
            Modal::Settings => self.render_settings_modal(cx),
            Modal::Help => self.render_help_modal(cx),
        };

        Some(
            deferred(
                div()
                    .occlude()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(rgba(0x05070a << 8 | 0xbb))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, _, cx| this.close_modal(cx)),
                    )
                    .child(panel),
            )
            .with_priority(10)
            .into_any_element(),
        )
    }
}
