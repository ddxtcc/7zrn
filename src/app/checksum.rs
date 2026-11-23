use std::time::{SystemTime, UNIX_EPOCH};

use gpui::Context;

use crate::state::{ChecksumSource, PendingAction};

use super::AppView;

impl AppView {
    pub(crate) fn checksum_fs_selection(&mut self, cx: &mut Context<Self>) {
        let paths = self.fs_action_paths();
        if paths.is_empty() {
            self.set_message("Select files to hash", true);
            cx.notify();
            return;
        }
        self.execute_action(
            PendingAction::Checksums {
                source: ChecksumSource::Fs { paths },
            },
            None,
            cx,
        );
    }

    pub(crate) fn checksum_archive_selection(&mut self, cx: &mut Context<Self>) {
        let keys = self.arc_action_keys();
        if keys.is_empty() {
            self.set_message("Select archive entries to hash", true);
            cx.notify();
            return;
        }
        let Some(archive) = self.archive.archive.clone() else {
            self.set_message("Open an archive first", true);
            cx.notify();
            return;
        };
        let Some(format) = self.archive.format else {
            return;
        };
        let raw_names = self.archive.expand_raw(&keys);
        if raw_names.is_empty() {
            return;
        }
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dest = crate::fsops::private_cache_root()
            .join("checksum")
            .join(stamp.to_string());
        self.execute_action(
            PendingAction::Checksums {
                source: ChecksumSource::Archive {
                    archive,
                    format,
                    dest,
                    raw_names,
                },
            },
            None,
            cx,
        );
    }

    pub(crate) fn verify_checksums(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.fs_active_key() else {
            self.set_message("Select a checksum file", true);
            cx.notify();
            return;
        };
        if !crate::checksum::looks_like_manifest(&path) {
            self.set_message("Select a SHA256SUMS, .sha256, .sha1, or .md5 file", true);
            cx.notify();
            return;
        }
        self.start_job(
            crate::job::JobKind::Verify,
            "Verifying checksums".to_string(),
            move |job| {
                let verification = crate::checksum::verify_manifest(&path)?;
                crate::job::note(&job, verification.summary());
                Ok(crate::job::JobPayload::Verified {
                    summary: verification.summary(),
                    failed: verification.failed,
                    missing: verification.missing,
                })
            },
            cx,
        );
    }

    pub(crate) fn verify_archive_checksums(&mut self, cx: &mut Context<Self>) {
        let keys = self.arc_action_keys();
        let Some(checksum_key) = keys
            .iter()
            .find(|key| crate::checksum::looks_like_manifest(std::path::Path::new(key)))
            .cloned()
        else {
            self.set_message("Select a checksum manifest inside the archive", true);
            cx.notify();
            return;
        };
        let Some(archive) = self.archive.archive.clone() else {
            return;
        };
        let Some(format) = self.archive.format else {
            return;
        };
        let Some(listing) = self.archive.listing.as_ref() else {
            return;
        };
        let entries: Vec<(String, String)> = listing
            .entries
            .iter()
            .map(|entry| (entry.display_path.clone(), entry.raw_path.clone()))
            .collect();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dest = crate::fsops::private_cache_root()
            .join("verify")
            .join(stamp.to_string());
        self.execute_action(
            PendingAction::VerifyChecksums {
                archive,
                format,
                dest,
                checksum_key,
                entries,
            },
            None,
            cx,
        );
    }

    pub(crate) fn can_verify_archive(&self) -> bool {
        self.arc_action_keys()
            .iter()
            .any(|key| crate::checksum::looks_like_manifest(std::path::Path::new(key)))
    }

    pub(crate) fn can_verify_active(&self) -> bool {
        self.fs_active_key()
            .is_some_and(|path| crate::checksum::looks_like_manifest(&path))
    }

    pub(crate) fn add_checksums_to_archive(
        &mut self,
        entries: &[crate::checksum::ChecksumEntry],
        cx: &mut Context<Self>,
    ) {
        if self.archive.archive.is_none() {
            self.set_message("Open an archive first", true);
            cx.notify();
            return;
        }
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let directory =
            std::env::temp_dir().join(format!("7zrn-checksums-{}-{stamp}", std::process::id()));
        if let Err(error) = std::fs::create_dir_all(&directory) {
            self.set_message(
                format!("Could not create temporary directory: {error}"),
                true,
            );
            cx.notify();
            return;
        }
        let path = directory.join("SHA256SUMS");
        if let Err(error) = std::fs::write(&path, crate::checksum::manifest(entries)) {
            self.set_message(format!("Could not write SHA256SUMS: {error}"), true);
            cx.notify();
            return;
        }
        self.modal = None;
        self.edit = None;
        self.add_paths(vec![path], cx);
    }

    pub(crate) fn save_checksums(
        &mut self,
        entries: &[crate::checksum::ChecksumEntry],
        cx: &mut Context<Self>,
    ) {
        let path = self.fs.path.join("SHA256SUMS");
        if path.exists() {
            self.set_message(format!("{} already exists", path.display()), true);
            cx.notify();
            return;
        }
        let manifest = crate::checksum::manifest(entries);
        match std::fs::write(&path, manifest) {
            Ok(()) => {
                self.set_message(format!("Wrote {}", path.display()), false);
                self.refresh_fs(cx);
            }
            Err(error) => {
                self.set_message(format!("Could not write {}: {error}", path.display()), true);
            }
        }
        cx.notify();
    }
}
