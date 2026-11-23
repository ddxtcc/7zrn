use gpui::prelude::*;
use gpui::{
    AnyElement, Context, IntoElement, MouseButton, MouseDownEvent, deferred, div, px, rgb, rgba,
};

use crate::app::AppView;
use crate::archive::NewFormat;
use crate::settings::Setting;
use crate::state::{ArchiveEditKind, EditTarget, Modal, SearchScope};
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
                compression,
                password,
                split,
                excludes,
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
                let password_text = match &self.edit {
                    Some(edit) if edit.target == EditTarget::NewPassword => edit.text.clone(),
                    _ => password.clone(),
                };
                let split_text = match &self.edit {
                    Some(edit) if edit.target == EditTarget::NewSplit => edit.text.clone(),
                    _ => split.clone(),
                };
                let excludes_text = match &self.edit {
                    Some(edit) if edit.target == EditTarget::NewExclude => edit.text.clone(),
                    _ => excludes.clone(),
                };
                let password_active = self
                    .edit
                    .as_ref()
                    .is_some_and(|edit| edit.target == EditTarget::NewPassword);
                let split_active = self
                    .edit
                    .as_ref()
                    .is_some_and(|edit| edit.target == EditTarget::NewSplit);
                let excludes_active = self
                    .edit
                    .as_ref()
                    .is_some_and(|edit| edit.target == EditTarget::NewExclude);
                let password_cursor = self
                    .edit
                    .as_ref()
                    .filter(|edit| edit.target == EditTarget::NewPassword)
                    .map(|edit| edit.cursor)
                    .unwrap_or(0);
                let split_cursor = self
                    .edit
                    .as_ref()
                    .filter(|edit| edit.target == EditTarget::NewSplit)
                    .map(|edit| edit.cursor)
                    .unwrap_or(0);
                let excludes_cursor = self
                    .edit
                    .as_ref()
                    .filter(|edit| edit.target == EditTarget::NewExclude)
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
                let mut levels = div().flex().flex_wrap().gap_2();
                for (level, label, id) in [
                    (0u8, "Store", "new-level-store"),
                    (1, "Fast", "new-level-fast"),
                    (5, "Normal", "new-level-normal"),
                    (9, "Max", "new-level-max"),
                ] {
                    levels = levels.child(self.chip(
                        id,
                        label,
                        *compression == level,
                        cx,
                        move |this, _, cx| this.set_new_compression(level, cx),
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
                        .child(div().text_xs().text_color(rgb(theme.dim)).child("Format"))
                        .child(formats)
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme.dim))
                                .child("Compression"),
                        )
                        .child(levels)
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme.dim))
                                .child("Password (7z and zip only)"),
                        )
                        .child(self.input_field(
                            InputField {
                                id: "new-password",
                                text: &password_text,
                                placeholder: "Optional password",
                                active: password_active,
                                cursor: password_cursor,
                                width: None,
                                target: Some(EditTarget::NewPassword),
                                click_text: password_text.clone(),
                            },
                            cx,
                        ))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme.dim))
                                .child("Split volumes (7z only, e.g. 100m)"),
                        )
                        .child(self.input_field(
                            InputField {
                                id: "new-split",
                                text: &split_text,
                                placeholder: "Optional volume size",
                                active: split_active,
                                cursor: split_cursor,
                                width: None,
                                target: Some(EditTarget::NewSplit),
                                click_text: split_text.clone(),
                            },
                            cx,
                        ))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme.dim))
                                .child("Exclude patterns, comma separated"),
                        )
                        .child(self.input_field(
                            InputField {
                                id: "new-excludes",
                                text: &excludes_text,
                                placeholder: "e.g. *.tmp, build",
                                active: excludes_active,
                                cursor: excludes_cursor,
                                width: None,
                                target: Some(EditTarget::NewExclude),
                                click_text: excludes_text.clone(),
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
                        .child(self.btn("new-cancel", "Cancel", true, cx, |this, _, cx| {
                            this.close_modal(cx)
                        }))
                        .child(self.primary_btn(
                            "new-create",
                            "Create",
                            false,
                            cx,
                            |this, window, cx| this.submit_modal(window, cx),
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
                            |this, window, cx| this.submit_modal(window, cx),
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
                            |this, window, cx| this.submit_modal(window, cx),
                        )),
                    cx,
                )
            }
            Modal::ArchiveEdit {
                title,
                kind,
                value,
                error,
            } => {
                let value_text = match &self.edit {
                    Some(edit) if edit.target == EditTarget::ArchiveEdit => edit.text.clone(),
                    _ => value.clone(),
                };
                let active = self
                    .edit
                    .as_ref()
                    .map(|edit| edit.target == EditTarget::ArchiveEdit)
                    .unwrap_or(false);
                let cursor = self
                    .edit
                    .as_ref()
                    .filter(|edit| edit.target == EditTarget::ArchiveEdit)
                    .map(|edit| edit.cursor)
                    .unwrap_or(0);
                let (hint, action) = match kind {
                    ArchiveEditKind::Rename { from } => (
                        format!("Rename {from}. Enter a new name in the same folder."),
                        "Rename",
                    ),
                    ArchiveEditKind::Copy { keys } => (
                        format!(
                            "Copy {} item(s) into a folder. Leave empty for the archive root.",
                            keys.len()
                        ),
                        "Copy",
                    ),
                    ArchiveEditKind::Move { keys } => (
                        format!(
                            "Move {} item(s) into a folder. Leave empty for the archive root.",
                            keys.len()
                        ),
                        "Move",
                    ),
                    ArchiveEditKind::NewFolder => (
                        "Create a folder. Use slashes for nested folders.".to_string(),
                        "Create",
                    ),
                };
                self.modal_panel(
                    title,
                    560.0,
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(div().text_sm().text_color(rgb(theme.muted)).child(hint))
                        .child(self.input_field(
                            InputField {
                                id: "archive-edit-input",
                                text: &value_text,
                                placeholder: "Name or path",
                                active,
                                cursor,
                                width: None,
                                target: Some(EditTarget::ArchiveEdit),
                                click_text: value_text.clone(),
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
                        .child(self.btn(
                            "archive-edit-cancel",
                            "Cancel",
                            true,
                            cx,
                            |this, _, cx| this.close_modal(cx),
                        ))
                        .child(self.primary_btn(
                            "archive-edit-submit",
                            action,
                            false,
                            cx,
                            |this, window, cx| this.submit_modal(window, cx),
                        )),
                    cx,
                )
            }
            Modal::Search {
                scope,
                query,
                content,
                results,
                selected,
                error,
            } => {
                let query_text = match &self.edit {
                    Some(edit) if edit.target == EditTarget::Search => edit.text.clone(),
                    _ => query.clone(),
                };
                let active = self
                    .edit
                    .as_ref()
                    .map(|edit| edit.target == EditTarget::Search)
                    .unwrap_or(false);
                let cursor = self
                    .edit
                    .as_ref()
                    .filter(|edit| edit.target == EditTarget::Search)
                    .map(|edit| edit.cursor)
                    .unwrap_or(0);
                let mut rows = div().flex().flex_col().gap_1();
                if results.is_empty() {
                    rows = rows.child(div().text_sm().text_color(rgb(theme.dim)).child(
                        if query.trim().is_empty() {
                            "Type to search the current tree."
                        } else {
                            "No matches."
                        },
                    ));
                }
                for (index, hit) in results.iter().enumerate() {
                    let hit_label = match hit {
                        crate::state::SearchHit::Fs { label, .. }
                        | crate::state::SearchHit::Archive { label, .. } => label.clone(),
                    };
                    let hit_value = hit.clone();
                    let row_selected = index == *selected;
                    rows = rows.child(
                        div()
                            .id(("search-hit", index))
                            .h(px(26.))
                            .w_full()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .rounded(px(3.))
                            .cursor_pointer()
                            .bg(rgb(if row_selected {
                                theme.select
                            } else {
                                theme.panel2
                            }))
                            .hover(move |style| style.bg(rgb(theme.hover)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.activate_search_hit(hit_value.clone(), cx);
                            }))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(if row_selected {
                                        theme.text
                                    } else {
                                        theme.muted
                                    }))
                                    .truncate()
                                    .child(hit_label),
                            ),
                    );
                }
                self.modal_panel(
                    "Search",
                    680.0,
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(self.input_field(
                            InputField {
                                id: "search-input",
                                text: &query_text,
                                placeholder: "Search names or text",
                                active,
                                cursor,
                                width: None,
                                target: Some(EditTarget::Search),
                                click_text: query_text.clone(),
                            },
                            cx,
                        ))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(self.chip(
                                    "search-files",
                                    "Files",
                                    *scope == SearchScope::Fs,
                                    cx,
                                    |this, _, cx| this.set_search_scope(SearchScope::Fs, cx),
                                ))
                                .child(self.chip(
                                    "search-archive",
                                    "Archive",
                                    *scope == SearchScope::Archive,
                                    cx,
                                    |this, _, cx| this.set_search_scope(SearchScope::Archive, cx),
                                ))
                                .child(self.chip(
                                    "search-content",
                                    "Contents",
                                    *content,
                                    cx,
                                    |this, _, cx| this.toggle_search_content(cx),
                                )),
                        )
                        .child(
                            div()
                                .id("search-results")
                                .max_h(px(300.))
                                .min_h(px(0.))
                                .overflow_y_scroll()
                                .child(rows),
                        )
                        .when_some(error.clone(), |this, error| {
                            this.child(div().text_xs().text_color(rgb(theme.red)).child(error))
                        }),
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            self.btn("search-cancel", "Cancel", true, cx, |this, _, cx| {
                                this.close_modal(cx)
                            }),
                        )
                        .child(self.btn("search-run", "Search", true, cx, |this, _, cx| {
                            this.run_search(cx)
                        }))
                        .child(self.primary_btn(
                            "search-open",
                            "Open",
                            false,
                            cx,
                            |this, window, cx| this.submit_modal(window, cx),
                        )),
                    cx,
                )
            }
            Modal::Palette { query, selected } => {
                let query_text = match &self.edit {
                    Some(edit) if edit.target == EditTarget::Palette => edit.text.clone(),
                    _ => query.clone(),
                };
                let active = self
                    .edit
                    .as_ref()
                    .map(|edit| edit.target == EditTarget::Palette)
                    .unwrap_or(false);
                let cursor = self
                    .edit
                    .as_ref()
                    .filter(|edit| edit.target == EditTarget::Palette)
                    .map(|edit| edit.cursor)
                    .unwrap_or(0);
                let actions = crate::palette::matches(&query_text);
                let mut rows = div().flex().flex_col().gap_1();
                for (index, action) in actions.iter().enumerate() {
                    let action = *action;
                    let row_selected = index == *selected;
                    rows = rows.child(
                        div()
                            .id(("palette-action", index))
                            .h(px(28.))
                            .w_full()
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .rounded(px(3.))
                            .cursor_pointer()
                            .bg(rgb(if row_selected {
                                theme.select
                            } else {
                                theme.panel2
                            }))
                            .hover(move |style| style.bg(rgb(theme.hover)))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.activate_palette_action(action, window, cx);
                            }))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .text_xs()
                                    .text_color(rgb(if row_selected {
                                        theme.text
                                    } else {
                                        theme.muted
                                    }))
                                    .truncate()
                                    .child(action.label()),
                            )
                            .child(
                                div()
                                    .flex_none()
                                    .text_xs()
                                    .text_color(rgb(theme.dim))
                                    .child(action.hint()),
                            ),
                    );
                }
                self.modal_panel(
                    "Command palette",
                    640.0,
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(self.input_field(
                            InputField {
                                id: "palette-input",
                                text: &query_text,
                                placeholder: "Type a command",
                                active,
                                cursor,
                                width: None,
                                target: Some(EditTarget::Palette),
                                click_text: query_text.clone(),
                            },
                            cx,
                        ))
                        .child(
                            div()
                                .id("palette-results")
                                .max_h(px(360.))
                                .min_h(px(0.))
                                .overflow_y_scroll()
                                .child(rows),
                        ),
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            self.btn("palette-cancel", "Cancel", true, cx, |this, _, cx| {
                                this.close_modal(cx)
                            }),
                        )
                        .child(self.primary_btn(
                            "palette-run",
                            "Run",
                            false,
                            cx,
                            |this, window, cx| this.submit_modal(window, cx),
                        )),
                    cx,
                )
            }
            Modal::ArchiveProperties { stats } => self.render_archive_properties_modal(stats, cx),
            Modal::TestPassed { stats } => self.render_test_passed_modal(stats, cx),
            Modal::Checksums { entries } => self.render_checksums_modal(entries, cx),
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
                            |this, window, cx| this.submit_modal(window, cx),
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
