use std::ffi::OsString;
use std::path::PathBuf;

use gpui::Context;

use crate::archive::{self, NewFormat};
use crate::fsops;
use crate::job::{self, JobKind, JobPayload};
use crate::state::*;

use super::AppView;

impl AppView {
    pub(crate) fn start_new_archive(
        &mut self,
        format: NewFormat,
        names: Vec<OsString>,
        base: PathBuf,
        name: String,
        options: archive::CreateOptions,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if names.is_empty() {
            return Err("Nothing to archive".into());
        }
        let name = name.trim();
        if name.is_empty() {
            return Err("Enter an archive path".into());
        }
        let mut path = fsops::resolve_path(name, &base);
        let ext = format.ext();
        let name_lower = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !name_lower.ends_with(&format!(".{ext}")) {
            let new_name = format!(
                "{}.{ext}",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("archive")
            );
            path = path.with_file_name(new_name);
        }
        if path.exists() {
            return Err(format!("{} already exists", path.display()));
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| base.clone());
        if !parent.is_dir() {
            return Err(format!("Directory does not exist: {}", parent.display()));
        }
        let label = format!("Creating {}", path.display());
        self.start_job(
            JobKind::Create,
            label,
            move |job| {
                let mut cmd = archive::create_command(&path, format, &base, &names, &options)?;
                job::run_command(&mut cmd, &job)?;
                let created = if path.exists() {
                    path.clone()
                } else {
                    let file_name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("archive");
                    path.with_file_name(format!("{file_name}.001"))
                };
                Ok(JobPayload::Created { archive: created })
            },
            cx,
        );
        Ok(())
    }

    pub(crate) fn start_extract(
        &mut self,
        dest: String,
        selected_only: bool,
        count_selected: usize,
        count_total: usize,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let Some(archive_path) = self.archive.archive.clone() else {
            return Err("No archive is open".into());
        };
        let Some(format) = self.archive.format else {
            return Err("No archive is open".into());
        };
        let dest = dest.trim();
        if dest.is_empty() {
            return Err("Enter a destination directory".into());
        }
        let dest = fsops::resolve_path(dest, &self.fs.path);
        if dest.exists() && !dest.is_dir() {
            return Err(format!("{} is not a directory", dest.display()));
        }
        std::fs::create_dir_all(&dest)
            .map_err(|e| format!("Could not create {}: {e}", dest.display()))?;

        let keys = self.arc_action_keys();
        let (raw_names, count, names) = if selected_only {
            if keys.is_empty() {
                return Err("Nothing selected".into());
            }
            let raw_names = if format.is_tar_family() {
                let listing = self
                    .archive
                    .listing
                    .as_ref()
                    .ok_or_else(|| "Archive is not loaded".to_string())?;
                archive::tar_extract_names(&listing.entries, &keys)
            } else {
                self.archive.expand_raw(&keys)
            };
            if raw_names.is_empty() {
                return Err("Nothing selected".into());
            }
            let count = if count_selected > 0 {
                count_selected
            } else {
                keys.len()
            };
            (raw_names, count, keys)
        } else {
            let listing = self
                .archive
                .listing
                .as_ref()
                .ok_or_else(|| "Archive is not loaded".to_string())?;
            let mut names: Vec<String> = archive::children(&listing.entries, "")
                .into_iter()
                .map(|child| child.name)
                .collect();
            names.sort();
            names.dedup();
            (Vec::new(), count_total, names)
        };
        self.execute_action(
            PendingAction::Extract {
                archive: archive_path,
                format,
                dest,
                raw_names,
                count,
                names,
            },
            None,
            cx,
        );
        Ok(())
    }

    pub(crate) fn start_extract_keys(
        &mut self,
        dest: PathBuf,
        keys: Vec<String>,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let Some(archive_path) = self.archive.archive.clone() else {
            return Err("No archive is open".into());
        };
        let Some(format) = self.archive.format else {
            return Err("No archive is open".into());
        };
        if keys.is_empty() {
            return Err("Nothing selected".into());
        }
        if dest.exists() && !dest.is_dir() {
            return Err(format!("{} is not a directory", dest.display()));
        }
        std::fs::create_dir_all(&dest)
            .map_err(|e| format!("Could not create {}: {e}", dest.display()))?;
        let raw_names = if format.is_tar_family() {
            let listing = self
                .archive
                .listing
                .as_ref()
                .ok_or_else(|| "Archive is not loaded".to_string())?;
            archive::tar_extract_names(&listing.entries, &keys)
        } else {
            self.archive.expand_raw(&keys)
        };
        if raw_names.is_empty() {
            return Err("Nothing selected".into());
        }
        let count = keys.len();
        self.execute_action(
            PendingAction::Extract {
                archive: archive_path,
                format,
                dest,
                raw_names,
                count,
                names: keys,
            },
            None,
            cx,
        );
        Ok(())
    }

    pub(crate) fn start_delete(
        &mut self,
        raw_names: Vec<String>,
        display_names: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(archive_path) = self.archive.archive.clone() else {
            return;
        };
        let Some(format) = self.archive.format else {
            return;
        };
        self.execute_action(
            PendingAction::Delete {
                archive: archive_path,
                format,
                raw_names,
                display_names,
            },
            None,
            cx,
        );
    }

    pub(crate) fn add_to_archive(&mut self, cx: &mut Context<Self>) {
        let paths = self.fs_action_paths();
        if paths.is_empty() {
            self.set_message("Select files or folders to add", true);
            cx.notify();
            return;
        }
        self.add_paths(paths, cx);
    }

    pub(crate) fn add_paths(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        self.add_paths_with_mode(paths, false, cx);
    }

    pub(crate) fn add_paths_with_mode(
        &mut self,
        paths: Vec<PathBuf>,
        remove_sources: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(archive_path) = self.archive.archive.clone() else {
            self.set_message("Open an archive first", true);
            cx.notify();
            return;
        };
        let Some(format) = self.archive.format else {
            return;
        };
        if self.archive.listing.is_none() {
            self.set_message("Open a readable archive first", true);
            cx.notify();
            return;
        }
        if !format.can_modify() {
            self.set_message(format!("{} archives are read-only", format.label()), true);
            cx.notify();
            return;
        }
        let paths: Vec<PathBuf> = paths.into_iter().filter(|path| path.exists()).collect();
        if paths.is_empty() {
            self.set_message("Nothing to add", true);
            cx.notify();
            return;
        }
        self.execute_action(
            PendingAction::Add {
                archive: archive_path,
                format,
                paths,
                remove_sources,
            },
            None,
            cx,
        );
    }

    pub(crate) fn move_to_archive(&mut self, cx: &mut Context<Self>) {
        let paths = self.fs_action_paths();
        if paths.is_empty() {
            self.set_message("Select files or folders to move", true);
            cx.notify();
            return;
        }
        let Some(format) = self.archive.format else {
            self.set_message("Open an archive first", true);
            cx.notify();
            return;
        };
        if !format.can_modify() || self.archive.archive.is_none() {
            self.set_message(format!("{} archives are read-only", format.label()), true);
            cx.notify();
            return;
        }
        self.modal = Some(Modal::MoveToArchive { paths });
        self.edit = None;
        cx.notify();
    }

    pub(crate) fn move_archive_to_fs(&mut self, cx: &mut Context<Self>) {
        let keys = self.arc_action_keys();
        if keys.is_empty() {
            self.set_message("Select archive entries to move", true);
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
        let raw_names = self.archive.expand_raw(&keys);
        if raw_names.is_empty() {
            return;
        }
        self.modal = Some(Modal::MoveOut {
            raw_names,
            names: keys,
            dest: self.fs.path.clone(),
        });
        self.edit = None;
        cx.notify();
    }

    pub(crate) fn start_move_out(
        &mut self,
        raw_names: Vec<String>,
        names: Vec<String>,
        dest: PathBuf,
        cx: &mut Context<Self>,
    ) {
        let Some(archive) = self.archive.archive.clone() else {
            return;
        };
        let Some(format) = self.archive.format else {
            return;
        };
        self.execute_action(
            PendingAction::MoveOut {
                archive,
                format,
                dest,
                raw_names,
                names,
            },
            None,
            cx,
        );
    }

    pub(crate) fn test_archive(&mut self, cx: &mut Context<Self>) {
        let Some(archive_path) = self.archive.archive.clone() else {
            self.set_message("Open an archive first", true);
            cx.notify();
            return;
        };
        let Some(format) = self.archive.format else {
            return;
        };
        if self.archive.listing.is_none() {
            self.set_message("Open a readable archive first", true);
            cx.notify();
            return;
        }
        self.execute_action(
            PendingAction::Test {
                archive: archive_path,
                format,
            },
            None,
            cx,
        );
    }
}
