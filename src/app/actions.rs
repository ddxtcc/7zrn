use std::path::PathBuf;
use std::time::Duration;

use gpui::{Context, Timer};

use crate::archive::{self, Format};
use crate::fsops;
use crate::job::{self, JobKind, JobPayload, SharedJob};
use crate::state::*;

use super::AppView;

impl AppView {
    pub(crate) fn open_archive(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if self.job.is_some() {
            self.set_message("Wait for the current operation to finish", true);
            cx.notify();
            return;
        }
        let format = Format::detect(&path);
        self.archive.clear();
        self.archive.archive = Some(path.clone());
        self.archive.format = Some(format);
        self.archive.password = None;
        self.archive.loading = true;
        self.archive.auto_nfo = true;
        self.focused = Pane::Archive;
        self.edit = None;
        self.push_recent(path.clone());
        self.execute_action(PendingAction::List { path }, None, cx);
        cx.notify();
    }

    pub(crate) fn open_archive_properties(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.archive.archive.clone() else {
            self.set_message("Open an archive first", true);
            cx.notify();
            return;
        };
        let Some(format) = self.archive.format else {
            return;
        };
        let Some(listing) = self.archive.listing.as_ref() else {
            self.set_message("Archive is still loading", true);
            cx.notify();
            return;
        };
        let stats =
            archive::ArchiveStats::build(&path, format, listing, self.archive.password.is_some());
        self.modal = Some(Modal::ArchiveProperties { stats });
        self.edit = None;
        cx.notify();
    }

    pub(crate) fn refresh_archive(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.archive.archive.clone() else {
            return;
        };
        if self.job.is_some() {
            return;
        }
        self.archive.loading = true;
        self.execute_action(PendingAction::List { path }, None, cx);
        cx.notify();
    }

    pub(crate) fn request_password(
        &mut self,
        action: PendingAction,
        error: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let message = format!(
            "Enter the password for {} to {}.",
            action.archive_path().display(),
            action.purpose()
        );
        self.modal = Some(Modal::Password {
            message,
            password: String::new(),
            error,
            action,
        });
        self.edit = Some(EditState {
            target: EditTarget::Password,
            text: String::new(),
            cursor: 0,
        });
        cx.notify();
    }

    pub(crate) fn execute_action(
        &mut self,
        action: PendingAction,
        password: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let needs_password =
            match &action {
                PendingAction::List { .. } => false,
                PendingAction::Checksums {
                    source: ChecksumSource::Fs { .. },
                } => false,
                _ => {
                    self.archive.password.is_none()
                        && self.archive.listing.as_ref().is_some_and(|listing| {
                            listing.entries.iter().any(|entry| entry.encrypted)
                        })
                }
            };
        if needs_password {
            self.request_password(action, None, cx);
            return;
        }
        if self.job.is_some() {
            self.set_message("Another operation is already running", true);
            cx.notify();
            return;
        }
        let password = password.or_else(|| self.archive.password.clone());
        if matches!(action, PendingAction::List { .. }) {
            self.archive.loading = true;
        }
        self.last_action = Some(action.clone());
        match action {
            PendingAction::List { path } => {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("archive")
                    .to_string();
                let label = format!("Reading {name}");
                let password = password.clone();
                self.start_job(
                    JobKind::List,
                    label,
                    move |job| {
                        job::note(&job, "Scanning archive...");
                        let listing = archive::list_archive(&path, password.as_deref())?;
                        Ok(JobPayload::Listed(listing))
                    },
                    cx,
                );
            }
            PendingAction::Extract {
                archive,
                format,
                dest,
                raw_names,
                count,
                names,
            } => {
                let label = format!("Extracting {count} item(s) to {}", dest.display());
                let password = password.clone();
                self.start_job(
                    JobKind::Extract,
                    label,
                    move |job| {
                        let mut cmd = archive::extract_command(
                            &archive,
                            format,
                            &dest,
                            &raw_names,
                            password.as_deref(),
                        )?;
                        job::run_command(&mut cmd, &job)?;
                        Ok(JobPayload::Extracted { dest, count, names })
                    },
                    cx,
                );
            }
            PendingAction::Test { archive, format } => {
                let label = "Testing archive".to_string();
                let password = password.clone();
                self.start_job(
                    JobKind::Test,
                    label,
                    move |job| {
                        let mut cmd = archive::test_command(&archive, format, password.as_deref());
                        let output = job::run_capture(&mut cmd, &job)?;
                        let summary = if format.is_tar_family() {
                            let count = output
                                .lines()
                                .filter(|line| !line.trim().is_empty())
                                .count();
                            format!("Archive is readable.\n{count} entries listed.")
                        } else {
                            let tail: Vec<&str> = output
                                .lines()
                                .map(str::trim)
                                .filter(|line| {
                                    !line.is_empty()
                                        && !line.chars().next().is_some_and(|c| c.is_ascii_digit())
                                })
                                .collect();
                            let tail = tail
                                .into_iter()
                                .rev()
                                .take(8)
                                .collect::<Vec<_>>()
                                .into_iter()
                                .rev()
                                .collect::<Vec<_>>()
                                .join("\n");
                            format!("No errors detected.\n\n{tail}")
                        };
                        Ok(JobPayload::Tested { summary })
                    },
                    cx,
                );
            }
            PendingAction::Open {
                archive,
                format,
                dest,
                raw_name,
                display_name,
            } => {
                let label = format!("Preparing {display_name}");
                let password = password.clone();
                self.start_job(
                    JobKind::Extract,
                    label,
                    move |job| {
                        std::fs::create_dir_all(&dest).map_err(|error| {
                            format!("Could not create {}: {error}", dest.display())
                        })?;
                        let raw_names = vec![raw_name];
                        let mut cmd = archive::extract_command(
                            &archive,
                            format,
                            &dest,
                            &raw_names,
                            password.as_deref(),
                        )?;
                        job::run_command(&mut cmd, &job)?;
                        let path = archive::find_extracted(&dest, &display_name)
                            .ok_or_else(|| format!("Could not find extracted {display_name}"))?;
                        Ok(JobPayload::Opened { path })
                    },
                    cx,
                );
            }
            PendingAction::Add {
                archive,
                format,
                paths,
            } => {
                let count = paths.len();
                let label = format!("Adding {count} item(s) to archive");
                let password = password.clone();
                self.start_job(
                    JobKind::Add,
                    label,
                    move |job| {
                        if format.is_tar_family() {
                            archive::tar_add(&archive, format, &paths, &job)?;
                        } else {
                            for mut cmd in
                                archive::add_commands(&archive, &paths, password.as_deref())?
                            {
                                job::run_command(&mut cmd, &job)?;
                            }
                        }
                        Ok(JobPayload::Modified)
                    },
                    cx,
                );
            }
            PendingAction::OpenNested {
                archive,
                format,
                dest,
                raw_name,
                display_name,
            } => {
                let label = format!("Opening nested {display_name}");
                let password = password.clone();
                let title = PathBuf::from(&display_name)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(&display_name)
                    .to_string();
                self.start_job(
                    JobKind::Extract,
                    label,
                    move |job| {
                        std::fs::create_dir_all(&dest).map_err(|error| {
                            format!("Could not create {}: {error}", dest.display())
                        })?;
                        let mut cmd = archive::extract_command(
                            &archive,
                            format,
                            &dest,
                            std::slice::from_ref(&raw_name),
                            password.as_deref(),
                        )?;
                        job::run_command(&mut cmd, &job)?;
                        let path =
                            archive::find_extracted(&dest, &display_name).ok_or_else(|| {
                                format!("Could not find nested archive {display_name}")
                            })?;
                        Ok(JobPayload::Nested { path, title })
                    },
                    cx,
                );
            }
            PendingAction::Edit {
                archive,
                format,
                edit,
            } => {
                let label = "Updating archive".to_string();
                let password = password.clone();
                self.start_job(
                    JobKind::Modify,
                    label,
                    move |job| {
                        archive::apply_edit(&archive, format, edit, password.as_deref(), &job)?;
                        Ok(JobPayload::Modified)
                    },
                    cx,
                );
            }
            PendingAction::Checksums { source } => {
                let label = "Computing checksums".to_string();
                let password = password.clone();
                self.start_job(
                    JobKind::Checksum,
                    label,
                    move |job| {
                        let entries = match source {
                            ChecksumSource::Fs { paths } => crate::checksum::hash_files(&paths)?,
                            ChecksumSource::Archive {
                                archive,
                                format,
                                dest,
                                raw_names,
                            } => {
                                std::fs::create_dir_all(&dest).map_err(|error| {
                                    format!("Could not create {}: {error}", dest.display())
                                })?;
                                let mut cmd = archive::extract_command(
                                    &archive,
                                    format,
                                    &dest,
                                    &raw_names,
                                    password.as_deref(),
                                )?;
                                job::run_command(&mut cmd, &job)?;
                                let entries = crate::checksum::hash_tree(&dest)?;
                                let _ = std::fs::remove_dir_all(&dest);
                                entries
                            }
                        };
                        job::note(&job, format!("Hashed {} file(s)", entries.len()));
                        Ok(JobPayload::Checksums(entries))
                    },
                    cx,
                );
            }
            PendingAction::Delete {
                archive,
                format,
                raw_names,
                display_names,
            } => {
                let label = format!("Deleting {} item(s)", display_names.len());
                let password = password.clone();
                self.start_job(
                    JobKind::Delete,
                    label,
                    move |job| {
                        if format.is_tar_family() {
                            archive::tar_delete(&archive, format, &display_names, &job)?;
                        } else {
                            let mut cmd =
                                archive::delete_command(&archive, &raw_names, password.as_deref())?;
                            job::run_command(&mut cmd, &job)?;
                        }
                        Ok(JobPayload::Modified)
                    },
                    cx,
                );
            }
        }
    }

    pub(crate) fn start_job<F>(
        &mut self,
        kind: JobKind,
        label: String,
        work: F,
        cx: &mut Context<Self>,
    ) where
        F: FnOnce(SharedJob) -> Result<JobPayload, String> + Send + 'static,
    {
        if self.job.is_some() {
            self.set_message("Another operation is already running", true);
            cx.notify();
            return;
        }
        let shared = job::new_job(kind, label);
        self.job = Some(shared.clone());
        std::thread::spawn({
            let shared = shared.clone();
            move || {
                let result = work(shared.clone());
                job::finish(&shared, result);
            }
        });

        cx.spawn(async move |this, cx| {
            loop {
                let done = shared.lock().unwrap().done;
                if done {
                    let _ = this.update(cx, |this, cx| this.finish_job(cx));
                    break;
                }
                let _ = this.update(cx, |_, cx| cx.notify());
                Timer::after(Duration::from_millis(120)).await;
            }
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn finish_job(&mut self, cx: &mut Context<Self>) {
        let Some(shared) = self.job.take() else {
            return;
        };
        let (kind, result) = {
            let mut job = shared.lock().unwrap();
            (job.kind, job.result.take())
        };
        match result {
            None => {
                self.last_action = None;
                self.set_message("Operation ended unexpectedly", true);
            }
            Some(Err(error)) => {
                if error == job::CANCELLED {
                    self.last_action = None;
                    if kind == JobKind::List {
                        self.archive.loading = false;
                    }
                    self.set_message("Operation cancelled", false);
                    cx.notify();
                    return;
                }
                if archive::password_required(&error)
                    && let Some(action) = self.last_action.take()
                {
                    self.archive.loading = false;
                    if kind == JobKind::List {
                        self.archive.error = Some(error.clone());
                    }
                    self.request_password(action, Some(error), cx);
                    return;
                }
                self.last_action = None;
                match kind {
                    JobKind::List => {
                        self.archive.loading = false;
                        self.archive.error = Some(error.clone());
                        self.set_message(error, true);
                    }
                    _ => {
                        self.set_message(error.clone(), true);
                        self.modal = Some(Modal::Info {
                            title: "Operation failed".into(),
                            body: error,
                            error: true,
                        });
                    }
                }
            }
            Some(Ok(payload)) => {
                self.last_action = None;
                match payload {
                    JobPayload::Listed(listing) => {
                        let counts = format!(
                            "{} files, {} folders - {}",
                            listing.file_count(),
                            listing.dir_count(),
                            fsops::human_size(listing.total_size())
                        );
                        let name = self
                            .archive
                            .archive
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("archive")
                            .to_string();
                        self.archive.listing = Some(listing);
                        self.archive.loading = false;
                        self.archive.error = None;
                        self.archive.fix_inner();
                        self.archive.prune_selection();
                        self.archive.clamp_active();
                        if self.archive.active.is_none() {
                            self.archive.reset_active();
                        }
                        if self.archive.auto_nfo {
                            self.archive.auto_nfo = false;
                            self.auto_preview_root_nfo(cx);
                        }
                        self.set_message(format!("{name} - {counts}"), false);
                    }
                    JobPayload::Opened { path } => {
                        self.open_path_external(path, cx);
                    }
                    JobPayload::Extracted { dest, count, names } => {
                        self.set_message(
                            format!("Extracted {count} item(s) to {}", dest.display()),
                            false,
                        );
                        if self.settings.open_after_extract {
                            self.reveal_extracted(dest, names, cx);
                        } else if dest == self.fs.path || dest.starts_with(&self.fs.path) {
                            self.refresh_fs(cx);
                        }
                    }
                    JobPayload::Modified => {
                        self.set_message("Archive updated", false);
                        self.refresh_archive(cx);
                    }
                    JobPayload::Created { archive } => {
                        self.set_message(format!("Created {}", archive.display()), false);
                        self.open_archive(archive, cx);
                    }
                    JobPayload::Nested { path, title } => {
                        self.set_message(format!("Opened nested archive {title}"), false);
                        self.open_archive_in_new_tab(path, cx);
                    }
                    JobPayload::Verified {
                        summary,
                        failed,
                        missing,
                    } => {
                        let clean = failed == 0 && missing == 0;
                        self.set_message(
                            if clean {
                                "Checksum verification passed"
                            } else {
                                "Checksum verification found problems"
                            },
                            !clean,
                        );
                        self.modal = Some(Modal::Info {
                            title: "Checksum verification".into(),
                            body: summary,
                            error: false,
                        });
                    }
                    JobPayload::Checksums(entries) => {
                        self.set_message(format!("Hashed {} file(s)", entries.len()), false);
                        self.modal = Some(Modal::Checksums { entries });
                    }
                    JobPayload::Tested { summary } => {
                        self.set_message("Archive test passed", false);
                        let stats = self
                            .archive
                            .archive
                            .as_ref()
                            .zip(self.archive.format.zip(self.archive.listing.as_ref()))
                            .map(|(path, (format, listing))| {
                                archive::ArchiveStats::build(
                                    path,
                                    format,
                                    listing,
                                    self.archive.password.is_some(),
                                )
                            });
                        self.modal = match stats {
                            Some(stats) => Some(Modal::TestPassed { stats }),
                            None => Some(Modal::Info {
                                title: "Archive test passed".into(),
                                body: summary,
                                error: false,
                            }),
                        };
                    }
                }
            }
        }
        cx.notify();
    }
}
