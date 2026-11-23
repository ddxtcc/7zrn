use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

use gpui::{ClipboardItem, Context, Timer};

use crate::archive::{self, Child, Format};
use crate::audio;
use crate::preview::{self, PreviewContent, PreviewLimits};
use crate::settings::AudioPlayer;
use crate::state::{Modal, PendingAction, PreviewView};

use super::AppView;

impl AppView {
    pub(crate) fn preview_fs_active(&mut self, cx: &mut Context<Self>) {
        let Some(active) = self.fs.active else {
            return;
        };
        let visible = self.fs.visible();
        let Some(entry_index) = visible.get(active).copied() else {
            return;
        };
        let entry = &self.fs.entries[entry_index];
        if entry.is_parent || entry.is_dir {
            self.set_message("Select a file to preview", false);
            cx.notify();
            return;
        }
        let path = entry.path.clone();
        let title = entry.name.clone();
        self.preview_fs_file(path, title, false, cx);
    }

    pub(crate) fn preview_fs_file(
        &mut self,
        path: PathBuf,
        title: String,
        automatic: bool,
        cx: &mut Context<Self>,
    ) {
        let size = std::fs::metadata(&path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        let limit = self.preview_limit(&path);
        if size > limit {
            if automatic {
                return;
            }
            self.show_preview(
                None,
                title,
                PreviewContent::Unsupported(limit_message(limit)),
                cx,
            );
            return;
        }
        self.show_preview(Some(path.clone()), title, PreviewContent::Loading, cx);
        let generation = self.preview_generation;
        let limits = self.preview_limits();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(preview::load(&path, limits));
        });
        cx.spawn(async move |this, cx| {
            loop {
                match receiver.try_recv() {
                    Ok(content) => {
                        let _ = this.update(cx, |this, cx| {
                            if this.preview_generation != generation {
                                return;
                            }
                            let audio = matches!(content, PreviewContent::Audio(_));
                            if let Some(preview) = &mut this.preview {
                                preview.content = content;
                            }
                            if audio && this.settings.audio_autoplay {
                                this.play_audio(cx);
                            }
                            cx.notify();
                        });
                        break;
                    }
                    Err(TryRecvError::Empty) => {
                        Timer::after(Duration::from_millis(40)).await;
                    }
                    Err(TryRecvError::Disconnected) => break,
                }
            }
        })
        .detach();
    }

    pub(crate) fn auto_preview_root_nfo(&mut self, cx: &mut Context<Self>) {
        if !self.settings.auto_preview
            || !self.settings.auto_nfo_preview
            || !self.archive.inner.is_empty()
        {
            return;
        }
        let children = self.arc_visible_children();
        let mut candidates: Vec<(usize, String)> = children
            .iter()
            .enumerate()
            .filter(|(_, child)| !child.is_parent && !child.is_dir)
            .filter_map(|(index, child)| {
                let name = child.name.to_ascii_lowercase();
                let rank = if name == "file_id.diz" {
                    0
                } else if name.ends_with(".nfo") {
                    1
                } else if name.ends_with(".diz") {
                    2
                } else {
                    return None;
                };
                Some((index, format!("{rank}:{}", child.name.to_ascii_lowercase())))
            })
            .collect();
        candidates.sort_by(|left, right| left.1.cmp(&right.1));
        let Some(index) = candidates.first().map(|(index, _)| *index) else {
            return;
        };
        let Some(child) = children.get(index).cloned() else {
            return;
        };
        let key = self.archive.key_for(&child.name);
        self.archive.active = Some(index);
        self.archive.anchor = Some(index);
        self.archive.selected.clear();
        self.archive.selected.insert(key);
        self.archive
            .scroll
            .scroll_to_item(index, gpui::ScrollStrategy::Center);
        self.preview_archive_entry(&child, true, cx);
        cx.notify();
    }

    pub(crate) fn open_nfo_full(&mut self, cx: &mut Context<Self>) {
        let Some(preview) = &self.preview else {
            return;
        };
        let PreviewContent::Nfo(document) = &preview.content else {
            return;
        };
        let document = document.clone();
        let title = preview.title.clone();
        self.modal = Some(Modal::Nfo { title, document });
        self.edit = None;
        cx.notify();
    }

    pub(crate) fn preview_archive_active(&mut self, cx: &mut Context<Self>) {
        let Some(active) = self.archive.active else {
            return;
        };
        let Some(child) = self.arc_visible_children().get(active).cloned() else {
            return;
        };
        if child.is_parent || child.is_dir {
            self.set_message("Select a file to preview", false);
            cx.notify();
            return;
        }
        self.preview_archive_entry(&child, false, cx);
    }

    pub(crate) fn preview_archive_entry(
        &mut self,
        child: &Child,
        automatic: bool,
        cx: &mut Context<Self>,
    ) {
        if child.is_parent || child.is_dir {
            return;
        }
        let key = self.archive.key_for(&child.name);
        let Some(listing) = self.archive.listing.as_ref() else {
            return;
        };
        let Some(entry) = listing
            .entries
            .iter()
            .find(|entry| entry.display_path == key)
        else {
            return;
        };
        let limit = self.archive_preview_limit(&key);
        if entry.size > limit {
            if automatic {
                return;
            }
            self.show_preview(
                None,
                key,
                PreviewContent::Unsupported(limit_message(limit)),
                cx,
            );
            return;
        }
        if entry.encrypted && self.archive.password.is_none() {
            if automatic {
                return;
            }
            self.show_preview(
                None,
                key,
                PreviewContent::Unsupported("Password required to preview this file.".into()),
                cx,
            );
            return;
        }
        let Some(archive) = self.archive.archive.clone() else {
            return;
        };
        let Some(format) = self.archive.format else {
            return;
        };
        let raw_name = entry.raw_path.clone();
        let password = self.archive.password.clone();
        let dest = preview_cache_dir(&archive);
        let limits = self.preview_limits();
        self.stop_audio();
        self.preview_generation += 1;
        let generation = self.preview_generation;
        self.context_menu = None;
        self.preview = Some(PreviewView {
            title: key.clone(),
            path: None,
            content: PreviewContent::Loading,
        });
        cx.notify();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(prepare_preview(
                archive, format, dest, raw_name, key, password, limits,
            ));
        });
        cx.spawn(async move |this, cx| {
            loop {
                match receiver.try_recv() {
                    Ok(result) => {
                        let _ = this.update(cx, |this, cx| {
                            if this.preview_generation != generation {
                                return;
                            }
                            match result {
                                Ok((path, content)) => {
                                    let audio = matches!(content, PreviewContent::Audio(_));
                                    if let Some(preview) = &mut this.preview {
                                        preview.path = Some(path);
                                        preview.content = content;
                                    }
                                    if audio && this.settings.audio_autoplay {
                                        this.play_audio(cx);
                                    }
                                }
                                Err(error) => {
                                    if let Some(preview) = &mut this.preview {
                                        preview.content = PreviewContent::Error(error);
                                    }
                                }
                            }
                            cx.notify();
                        });
                        break;
                    }
                    Err(TryRecvError::Empty) => {
                        Timer::after(Duration::from_millis(60)).await;
                    }
                    Err(TryRecvError::Disconnected) => break,
                }
            }
        })
        .detach();
    }

    pub(crate) fn open_archive_active(&mut self, cx: &mut Context<Self>) {
        let Some(active) = self.archive.active else {
            return;
        };
        let Some(child) = self.arc_visible_children().get(active).cloned() else {
            return;
        };
        if child.is_parent {
            self.archive_up(cx);
            return;
        }
        if child.is_dir {
            self.enter_archive_child(&child, cx);
            return;
        }
        let Some((archive, format, dest, raw_name, display_name)) = self.archive_file_parts(&child)
        else {
            return;
        };
        if archive::is_archive_name(&child.name) {
            let parent = dest.parent().map(PathBuf::from).unwrap_or(dest);
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0);
            let dest = parent.join(format!("nested-{stamp}"));
            self.execute_action(
                PendingAction::OpenNested {
                    archive,
                    format,
                    dest,
                    raw_name,
                    display_name,
                },
                None,
                cx,
            );
            return;
        }
        self.execute_action(
            PendingAction::Open {
                archive,
                format,
                dest,
                raw_name,
                display_name,
            },
            None,
            cx,
        );
    }

    fn archive_file_parts(
        &self,
        child: &Child,
    ) -> Option<(PathBuf, Format, PathBuf, String, String)> {
        let archive = self.archive.archive.clone()?;
        let format = self.archive.format?;
        let key = self.archive.key_for(&child.name);
        let listing = self.archive.listing.as_ref()?;
        let entry = listing
            .entries
            .iter()
            .find(|entry| entry.display_path == key)?;
        Some((
            archive.clone(),
            format,
            open_cache_dir(&archive),
            entry.raw_path.clone(),
            key,
        ))
    }

    pub(crate) fn open_archive_in_new_tab(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let parent = path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.fs.path.clone());
        self.new_tab(cx);
        self.fs = crate::state::FsState::new_with_hidden(parent, self.settings.show_hidden);
        self.open_archive(path, cx);
    }

    pub(crate) fn toggle_code_wrap(&mut self, cx: &mut Context<Self>) {
        self.code_wrap = !self.code_wrap;
        self.settings.code_wrap = self.code_wrap;
        if let Err(error) = self.settings.save() {
            self.settings_error = Some(error);
        }
        cx.notify();
    }

    pub(crate) fn open_url_external(&mut self, url: String, cx: &mut Context<Self>) {
        if !(url.starts_with("http://")
            || url.starts_with("https://")
            || url.starts_with("mailto:")
            || url.starts_with("file:"))
        {
            self.set_message("Unsupported link scheme", true);
            cx.notify();
            return;
        }
        let mut result = Command::new("xdg-open").arg(&url).spawn();
        if result.is_err() {
            result = Command::new("gio").arg("open").arg(&url).spawn();
        }
        match result {
            Ok(child) => {
                std::thread::spawn(move || {
                    let mut child = child;
                    let _ = child.wait();
                });
                self.set_message(format!("Opened {url}"), false);
            }
            Err(error) => self.set_message(format!("Could not open {url}: {error}"), true),
        }
        cx.notify();
    }

    pub(crate) fn copy_preview(&mut self, cx: &mut Context<Self>) {
        let Some(text) = self
            .preview
            .as_ref()
            .and_then(|preview| preview.content.copy_text())
        else {
            return;
        };
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.set_message("Copied preview text", false);
        cx.notify();
    }

    pub(crate) fn set_image_zoom(&mut self, delta: f32, cx: &mut Context<Self>) {
        self.image_zoom = (self.image_zoom + delta).clamp(0.1, 8.0);
        cx.notify();
    }

    pub(crate) fn play_audio(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self
            .preview
            .as_ref()
            .filter(|preview| matches!(preview.content, PreviewContent::Audio(_)))
            .and_then(|preview| preview.path.clone())
        else {
            return;
        };
        self.stop_audio();
        self.audio_generation = self.audio_generation.wrapping_add(1);
        let generation = self.audio_generation;
        let child = start_audio(&path, self.settings.audio_player);
        match child {
            Ok(child) => {
                self.audio = Some(child);
                self.set_message(format!("Playing {}", path.display()), false);
                cx.spawn(async move |this, cx| {
                    loop {
                        Timer::after(Duration::from_millis(300)).await;
                        let finished = this.update(cx, |this, cx| {
                            if this.audio_generation != generation {
                                return true;
                            }
                            let Some(child) = this.audio.as_mut() else {
                                return true;
                            };
                            match child.try_wait() {
                                Ok(Some(_)) | Err(_) => {
                                    this.audio = None;
                                    this.set_message("Audio finished", false);
                                    cx.notify();
                                    true
                                }
                                Ok(None) => false,
                            }
                        });
                        if !matches!(finished, Ok(false)) {
                            break;
                        }
                    }
                })
                .detach();
            }
            Err(error) => {
                self.set_message(
                    format!("Could not start audio playback: {error}. Install ffmpeg or mpv."),
                    true,
                );
            }
        }
        cx.notify();
    }

    pub(crate) fn stop_audio(&mut self) {
        self.audio_generation = self.audio_generation.wrapping_add(1);
        if let Some(mut child) = self.audio.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    pub(crate) fn show_preview(
        &mut self,
        path: Option<PathBuf>,
        title: String,
        content: PreviewContent,
        cx: &mut Context<Self>,
    ) {
        self.stop_audio();
        let is_audio = matches!(content, PreviewContent::Audio(_));
        self.preview_generation += 1;
        self.context_menu = None;
        if matches!(content, PreviewContent::Image { .. }) {
            self.image_zoom = 1.0;
        }
        self.preview = Some(PreviewView {
            title,
            path,
            content,
        });
        self.edit = None;
        if is_audio && self.settings.audio_autoplay {
            self.play_audio(cx);
        }
        cx.notify();
    }

    pub(crate) fn close_preview(&mut self, cx: &mut Context<Self>) {
        self.stop_audio();
        self.preview = None;
        self.preview_generation += 1;
        cx.notify();
    }

    pub(crate) fn open_preview_file(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self
            .preview
            .as_ref()
            .and_then(|preview| preview.path.clone())
        else {
            return;
        };
        self.open_path_external(path, cx);
    }

    pub(crate) fn open_path_external(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let mut result = Command::new("xdg-open").arg(&path).spawn();
        if result.is_err() {
            result = Command::new("gio").arg("open").arg(&path).spawn();
        }
        match result {
            Ok(child) => {
                std::thread::spawn(move || {
                    let mut child = child;
                    let _ = child.wait();
                });
                self.set_message(format!("Opened {}", path.display()), false);
            }
            Err(error) => {
                self.set_message(format!("Could not open {}: {error}", path.display()), true)
            }
        }
        cx.notify();
    }
}

impl AppView {
    fn preview_limits(&self) -> PreviewLimits {
        PreviewLimits {
            text_bytes: (self.settings.preview_text_kb * 1024) as usize,
            hex_bytes: (self.settings.preview_hex_kb * 1024) as usize,
            pdf_dpi: self.settings.pdf_dpi,
            audio_cover: self.settings.audio_cover_art,
        }
    }

    fn preview_limit(&self, path: &Path) -> u64 {
        Self::limit_bytes(if audio::is_audio(path) {
            self.settings.audio_limit_mb
        } else {
            self.settings.preview_limit_mb
        })
    }

    fn archive_preview_limit(&self, name: &str) -> u64 {
        Self::limit_bytes(if audio::is_audio(Path::new(name)) {
            self.settings.audio_limit_mb
        } else {
            self.settings.preview_limit_mb
        })
    }

    fn limit_bytes(mebibytes: u64) -> u64 {
        mebibytes.saturating_mul(1024 * 1024)
    }
}

fn limit_message(limit: u64) -> String {
    let mebibytes = limit / (1024 * 1024);
    format!("File is too large to preview (max {mebibytes} MiB).")
}

fn preview_cache_dir(archive: &Path) -> PathBuf {
    cache_dir("7zrn-preview", archive)
}

fn open_cache_dir(archive: &Path) -> PathBuf {
    cache_dir("7zrn-open", archive)
}

fn cache_dir(prefix: &str, archive: &Path) -> PathBuf {
    let name = archive
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("archive");
    crate::fsops::private_cache_root().join(prefix).join(name)
}

fn start_audio(path: &Path, preferred: AudioPlayer) -> std::io::Result<std::process::Child> {
    let order = match preferred {
        AudioPlayer::Ffplay => [AudioPlayer::Ffplay, AudioPlayer::Mpv],
        AudioPlayer::Mpv => [AudioPlayer::Mpv, AudioPlayer::Ffplay],
    };
    let mut last_error = None;
    for player in order {
        let mut command = std::process::Command::new(match player {
            AudioPlayer::Ffplay => "ffplay",
            AudioPlayer::Mpv => "mpv",
        });
        match player {
            AudioPlayer::Ffplay => {
                command.args(["-nodisp", "-autoexit", "-loglevel", "quiet"]);
            }
            AudioPlayer::Mpv => {
                command.args(["--no-video", "--really-quiet"]);
            }
        }
        command
            .arg(path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        match command.spawn() {
            Ok(child) => return Ok(child),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "No audio player available")
    }))
}

fn prepare_preview(
    archive: PathBuf,
    format: Format,
    dest: PathBuf,
    raw_name: String,
    display_name: String,
    password: Option<String>,
    limits: PreviewLimits,
) -> Result<(PathBuf, PreviewContent), String> {
    crate::fsops::ensure_private_dir(&dest)?;
    let raw_names = vec![raw_name];
    let command =
        archive::extract_command(&archive, format, &dest, &raw_names, password.as_deref())?;
    let output = archive::output_with_password(command, password.as_deref())
        .map_err(|error| format!("Could not extract preview: {error}"))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Could not extract {display_name}.\n{}",
            detail.trim()
        ));
    }
    let path = archive::find_extracted(&dest, &display_name)
        .ok_or_else(|| format!("Could not find extracted {display_name}"))?;
    let content = preview::load(&path, limits);
    Ok((path, content))
}
