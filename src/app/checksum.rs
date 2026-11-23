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
        let dest = std::env::temp_dir()
            .join("7zrn-checksum")
            .join(std::process::id().to_string())
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

    pub(crate) fn can_verify_active(&self) -> bool {
        self.fs_active_key()
            .is_some_and(|path| crate::checksum::looks_like_manifest(&path))
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
