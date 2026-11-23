use std::io::Read;
use std::path::PathBuf;

use gpui::Context;

use crate::job::{self, JobKind, JobPayload};

use super::AppView;

impl AppView {
    pub(crate) fn merge_volumes(&mut self, cx: &mut Context<Self>) {
        let Some((output, parts)) = self.volume_set() else {
            self.set_message("Select two or more numbered volumes", true);
            cx.notify();
            return;
        };
        if output.exists() {
            self.set_message(format!("{} already exists", output.display()), true);
            cx.notify();
            return;
        }
        let label = format!("Merging {} volume(s)", parts.len());
        self.start_job(
            JobKind::Create,
            label,
            move |job| {
                let mut out = std::fs::File::create(&output)
                    .map_err(|error| format!("Could not create {}: {error}", output.display()))?;
                let mut buffer = [0u8; 64 * 1024];
                for part in &parts {
                    job::note(&job, format!("Appending {}", part.display()));
                    let mut input = std::fs::File::open(part)
                        .map_err(|error| format!("Could not open {}: {error}", part.display()))?;
                    loop {
                        let read = input.read(&mut buffer).map_err(|error| {
                            format!("Could not read {}: {error}", part.display())
                        })?;
                        if read == 0 {
                            break;
                        }
                        std::io::Write::write_all(&mut out, &buffer[..read]).map_err(|error| {
                            format!("Could not write {}: {error}", output.display())
                        })?;
                    }
                }
                Ok(JobPayload::Created { archive: output })
            },
            cx,
        );
    }

    pub(crate) fn can_merge_volumes(&self) -> bool {
        self.volume_set().is_some()
    }

    fn volume_set(&self) -> Option<(PathBuf, Vec<PathBuf>)> {
        let mut selected: Vec<PathBuf> = self
            .fs_action_paths()
            .into_iter()
            .filter(|path| path.is_file())
            .collect();
        if selected.len() < 2 {
            return None;
        }
        selected.sort();
        let first = selected.first()?;
        let name = first.file_name()?.to_str()?;
        let (base, suffix) = name.rsplit_once('.')?;
        if !suffix.chars().all(|character| character.is_ascii_digit()) || suffix.len() < 3 {
            return None;
        }
        if !selected.iter().all(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.rsplit_once('.'))
                .is_some_and(|(path_base, path_suffix)| {
                    path_base == base
                        && path_suffix
                            .chars()
                            .all(|character| character.is_ascii_digit())
                })
        }) {
            return None;
        }
        let output = first.parent()?.join(base);
        Some((output, selected))
    }
}
