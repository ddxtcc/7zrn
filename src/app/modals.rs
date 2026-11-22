use std::io::ErrorKind;
use std::process::Command;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

use gpui::{ClipboardItem, Context, Timer, Window};
use std::ffi::OsString;

use crate::archive;
use crate::archive::NewFormat;
use crate::state::*;
use crate::text::replace_archive_extension;

use super::AppView;

impl AppView {
    pub(crate) fn open_new_archive_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = self.fs_action_paths();
        if paths.is_empty() {
            self.set_message("Select files or folders to archive", true);
            cx.notify();
            return;
        }
        let names: Vec<OsString> = paths
            .iter()
            .filter_map(|path| path.file_name().map(|name| name.to_os_string()))
            .collect();
        if names.is_empty() {
            return;
        }
        let base = self.fs.path.clone();
        let suggested = if paths.len() == 1 {
            let name = paths[0]
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("archive");
            format!("{name}.7z")
        } else {
            "archive.7z".to_string()
        };
        let name = base.join(suggested).display().to_string();
        self.modal = Some(Modal::NewArchive {
            format: NewFormat::SevenZ,
            names,
            base,
            name: name.clone(),
            compression: 5,
            password: String::new(),
            split: String::new(),
            excludes: String::new(),
            error: None,
        });
        self.begin_edit(EditTarget::NewName, name, window, cx);
    }

    pub(crate) fn open_extract_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(listing) = self.archive.listing.as_ref() else {
            self.set_message("Open a readable archive first", true);
            cx.notify();
            return;
        };
        let keys = self.arc_action_keys();
        let count_total = listing.file_count();
        let dest = self.fs.path.display().to_string();
        let selected_only = !keys.is_empty();
        self.modal = Some(Modal::Extract {
            dest: dest.clone(),
            selected_only,
            count_selected: keys.len(),
            count_total,
            error: None,
        });
        self.begin_edit(EditTarget::ExtractDest, dest, window, cx);
    }

    pub(crate) fn request_delete(&mut self, cx: &mut Context<Self>) {
        if self.archive.listing.is_none() {
            self.set_message("Open a readable archive first", true);
            cx.notify();
            return;
        }
        let Some(format) = self.archive.format else {
            return;
        };
        if !format.can_modify() {
            self.set_message(format!("{} archives are read-only", format.label()), true);
            cx.notify();
            return;
        }
        let keys = self.arc_action_keys();
        if keys.is_empty() {
            self.set_message("Select entries to delete", true);
            cx.notify();
            return;
        }
        let raw_names = self.archive.expand_raw(&keys);
        if raw_names.is_empty() {
            self.set_message("Select entries to delete", true);
            cx.notify();
            return;
        }
        if self.settings.confirm_delete {
            self.modal = Some(Modal::DeleteConfirm {
                raw_names,
                display_names: keys,
            });
            self.edit = None;
            cx.notify();
        } else {
            self.start_delete(raw_names, keys, cx);
        }
    }

    pub(crate) fn submit_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(edit) = self.edit.take() {
            self.write_edit_text_to_target(edit.target, edit.text);
        }
        if matches!(self.modal, Some(Modal::Search { .. })) {
            self.submit_search(cx);
            return;
        }
        if matches!(self.modal, Some(Modal::Palette { .. })) {
            self.submit_palette(window, cx);
            return;
        }
        let Some(mut modal) = self.modal.take() else {
            return;
        };
        match &mut modal {
            Modal::NewArchive {
                format,
                names,
                base,
                name,
                compression,
                password,
                split,
                excludes,
                error,
            } => {
                let options = crate::archive::CreateOptions {
                    compression: *compression,
                    password: (!password.is_empty()).then(|| password.clone()),
                    split: (!split.trim().is_empty()).then(|| split.trim().to_string()),
                    excludes: excludes
                        .split([',', '\n'])
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .collect(),
                };
                match self.start_new_archive(
                    *format,
                    names.clone(),
                    base.clone(),
                    name.clone(),
                    options,
                    cx,
                ) {
                    Ok(()) => {}
                    Err(message) => {
                        *error = Some(message);
                        let text = name.clone();
                        let cursor = text.len();
                        self.modal = Some(modal);
                        self.edit = Some(EditState {
                            target: EditTarget::NewName,
                            text,
                            cursor,
                        });
                    }
                }
            }
            Modal::Extract {
                dest,
                selected_only,
                count_selected,
                count_total,
                error,
            } => match self.start_extract(
                dest.clone(),
                *selected_only,
                *count_selected,
                *count_total,
                cx,
            ) {
                Ok(()) => {}
                Err(message) => {
                    *error = Some(message);
                    let text = dest.clone();
                    let cursor = text.len();
                    self.modal = Some(modal);
                    self.edit = Some(EditState {
                        target: EditTarget::ExtractDest,
                        text,
                        cursor,
                    });
                }
            },
            Modal::DeleteConfirm {
                raw_names,
                display_names,
            } => {
                self.start_delete(raw_names.clone(), display_names.clone(), cx);
            }
            Modal::ArchiveEdit {
                kind, value, error, ..
            } => {
                let value = value.trim().to_string();
                if value.is_empty() {
                    *error = Some("Enter a value".into());
                    let text = value;
                    let cursor = text.len();
                    self.modal = Some(modal);
                    self.edit = Some(EditState {
                        target: EditTarget::ArchiveEdit,
                        text,
                        cursor,
                    });
                } else {
                    let Some(archive) = self.archive.archive.clone() else {
                        self.modal = None;
                        self.set_message("Open an archive first", true);
                        cx.notify();
                        return;
                    };
                    let Some(format) = self.archive.format else {
                        self.modal = None;
                        self.set_message("Open an archive first", true);
                        cx.notify();
                        return;
                    };
                    let edit = match kind {
                        ArchiveEditKind::Rename { from } => {
                            if value.contains('/') {
                                *error = Some("Use a file name without a path".into());
                                let text = value;
                                let cursor = text.len();
                                self.modal = Some(modal);
                                self.edit = Some(EditState {
                                    target: EditTarget::ArchiveEdit,
                                    text,
                                    cursor,
                                });
                                cx.notify();
                                return;
                            }
                            let parent = from
                                .rsplit_once('/')
                                .map(|(parent, _)| parent)
                                .unwrap_or("");
                            let to = if parent.is_empty() {
                                value.clone()
                            } else {
                                format!("{parent}/{value}")
                            };
                            archive::ArchiveEdit::Rename {
                                from: from.clone(),
                                to,
                            }
                        }
                        ArchiveEditKind::Copy { keys } => archive::ArchiveEdit::Copy {
                            keys: keys.clone(),
                            dest: value.clone(),
                        },
                        ArchiveEditKind::Move { keys } => archive::ArchiveEdit::Move {
                            keys: keys.clone(),
                            dest: value.clone(),
                        },
                        ArchiveEditKind::NewFolder => archive::ArchiveEdit::NewFolder {
                            path: value.clone(),
                        },
                    };
                    self.modal = None;
                    self.edit = None;
                    self.execute_action(
                        PendingAction::Edit {
                            archive,
                            format,
                            edit,
                        },
                        None,
                        cx,
                    );
                }
            }
            Modal::Password {
                password,
                error,
                action,
                ..
            } => {
                if password.is_empty() {
                    *error = Some("Enter a password".into());
                    let text = password.clone();
                    self.modal = Some(modal);
                    self.edit = Some(EditState {
                        target: EditTarget::Password,
                        text,
                        cursor: 0,
                    });
                } else {
                    let password = password.clone();
                    let action = action.clone();
                    self.archive.password = Some(password.clone());
                    self.execute_action(action, Some(password), cx);
                }
            }
            Modal::Search { .. }
            | Modal::Palette { .. }
            | Modal::ArchiveProperties { .. }
            | Modal::TestPassed { .. }
            | Modal::Info { .. }
            | Modal::Settings
            | Modal::Help => {}
        }
        cx.notify();
    }

    fn archive_edit_modal(
        &mut self,
        title: &str,
        kind: ArchiveEditKind,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.modal = Some(Modal::ArchiveEdit {
            title: title.to_string(),
            kind,
            value: value.clone(),
            error: None,
        });
        self.begin_edit(EditTarget::ArchiveEdit, value, window, cx);
    }

    pub(crate) fn open_rename_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(from) = self.arc_action_keys().into_iter().next() else {
            self.set_message("Select an entry to rename", true);
            cx.notify();
            return;
        };
        let name = from.rsplit('/').next().unwrap_or(&from).to_string();
        self.archive_edit_modal(
            "Rename entry",
            ArchiveEditKind::Rename { from },
            name,
            window,
            cx,
        );
    }

    pub(crate) fn open_copy_move_modal(
        &mut self,
        copy: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let keys = self.arc_action_keys();
        if keys.is_empty() {
            self.set_message("Select entries first", true);
            cx.notify();
            return;
        }
        let title = if copy { "Copy entries" } else { "Move entries" };
        let kind = if copy {
            ArchiveEditKind::Copy { keys }
        } else {
            ArchiveEditKind::Move { keys }
        };
        self.archive_edit_modal(title, kind, self.archive.inner.clone(), window, cx);
    }

    pub(crate) fn open_new_folder_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let value = if self.archive.inner.is_empty() {
            String::new()
        } else {
            format!("{}/", self.archive.inner)
        };
        self.archive_edit_modal("New folder", ArchiveEditKind::NewFolder, value, window, cx);
    }

    pub(crate) fn set_new_compression(&mut self, compression: u8, cx: &mut Context<Self>) {
        if let Some(Modal::NewArchive {
            compression: current,
            ..
        }) = &mut self.modal
        {
            *current = compression;
        }
        cx.notify();
    }

    pub(crate) fn set_new_format(&mut self, format: NewFormat, cx: &mut Context<Self>) {
        let editing_name = self
            .edit
            .as_ref()
            .filter(|edit| edit.target == EditTarget::NewName)
            .map(|edit| edit.text.clone());
        if let Some(Modal::NewArchive {
            format: current,
            name,
            ..
        }) = &mut self.modal
        {
            *current = format;
            let base_text = editing_name.as_deref().unwrap_or(name.as_str());
            let new_name = replace_archive_extension(base_text, format.ext());
            *name = new_name.clone();
            if let Some(edit) = self.edit.as_mut()
                && edit.target == EditTarget::NewName
            {
                edit.text = new_name;
                edit.cursor = edit.text.len();
            }
        }
        cx.notify();
    }

    pub(crate) fn set_extract_scope(&mut self, selected_only: bool, cx: &mut Context<Self>) {
        if let Some(Modal::Extract {
            selected_only: current,
            ..
        }) = &mut self.modal
        {
            *current = selected_only;
        }
        cx.notify();
    }

    pub(crate) fn copy_modal_body(&mut self, cx: &mut Context<Self>) {
        let Some(Modal::Info { body, .. }) = &self.modal else {
            return;
        };
        let body = body.clone();
        cx.write_to_clipboard(ClipboardItem::new_string(body));
        self.set_message("Copied dialog text", false);
        cx.notify();
    }

    pub(crate) fn browse_extract_destination(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !matches!(self.modal, Some(Modal::Extract { .. })) {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(pick_folder());
        });
        cx.spawn(async move |this, cx| {
            loop {
                match receiver.try_recv() {
                    Ok(result) => {
                        let _ = this.update(cx, |this, cx| match result {
                            Ok(Some(path)) => {
                                if let Some(Modal::Extract { dest, error, .. }) = &mut this.modal {
                                    *dest = path.clone();
                                    *error = None;
                                }
                                if let Some(edit) = this
                                    .edit
                                    .as_mut()
                                    .filter(|edit| edit.target == EditTarget::ExtractDest)
                                {
                                    edit.text = path.clone();
                                    edit.cursor = path.len();
                                }
                                cx.notify();
                            }
                            Ok(None) => {}
                            Err(message) => {
                                this.set_message(message, true);
                                cx.notify();
                            }
                        });
                        break;
                    }
                    Err(TryRecvError::Empty) => {
                        Timer::after(Duration::from_millis(80)).await;
                    }
                    Err(TryRecvError::Disconnected) => break,
                }
            }
        })
        .detach();
    }

    pub(crate) fn close_modal(&mut self, cx: &mut Context<Self>) {
        self.modal = None;
        self.edit = None;
        cx.notify();
    }

    pub(crate) fn focus_modal_field(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(edit) = self.edit.take() {
            self.write_edit_text_to_target(edit.target, edit.text);
        }
        match &self.modal {
            Some(Modal::NewArchive {
                name,
                password,
                split,
                excludes,
                ..
            }) => {
                let current = self.edit.as_ref().map(|edit| edit.target);
                let next = match current {
                    Some(EditTarget::NewName) => EditTarget::NewPassword,
                    Some(EditTarget::NewPassword) => EditTarget::NewSplit,
                    Some(EditTarget::NewSplit) => EditTarget::NewExclude,
                    _ => EditTarget::NewName,
                };
                let text = match next {
                    EditTarget::NewPassword => password.clone(),
                    EditTarget::NewSplit => split.clone(),
                    EditTarget::NewExclude => excludes.clone(),
                    _ => name.clone(),
                };
                self.begin_edit(next, text, window, cx);
            }
            Some(Modal::Extract { dest, .. }) => {
                let text = dest.clone();
                self.begin_edit(EditTarget::ExtractDest, text, window, cx);
            }
            Some(Modal::Password { password, .. }) => {
                let text = password.clone();
                self.begin_edit(EditTarget::Password, text, window, cx);
            }
            Some(Modal::ArchiveEdit { value, .. }) => {
                let text = value.clone();
                self.begin_edit(EditTarget::ArchiveEdit, text, window, cx);
            }
            Some(Modal::Search { query, .. }) => {
                let text = query.clone();
                self.begin_edit(EditTarget::Search, text, window, cx);
            }
            Some(Modal::Palette { query, .. }) => {
                let text = query.clone();
                self.begin_edit(EditTarget::Palette, text, window, cx);
            }
            _ => {}
        }
    }

    pub(crate) fn close_modal_with_escape(&mut self, cx: &mut Context<Self>) {
        if self.edit.is_some() {
            self.edit = None;
            cx.notify();
        } else {
            self.close_modal(cx);
        }
    }
}

fn pick_folder() -> Result<Option<String>, String> {
    const PICKERS: [(&str, &[&str]); 3] = [
        (
            "zenity",
            &[
                "--file-selection",
                "--directory",
                "--title=Select extraction folder",
            ],
        ),
        (
            "yad",
            &["--file", "--directory", "--title=Select extraction folder"],
        ),
        ("kdialog", &["--getexistingdirectory"]),
    ];
    for (command, arguments) in PICKERS {
        match Command::new(command).args(arguments).output() {
            Ok(output) => {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return Ok((!path.is_empty()).then_some(path));
                }
                return Ok(None);
            }
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("Could not open folder picker: {error}")),
        }
    }
    Err("No folder picker found (install zenity, yad, or kdialog)".into())
}
