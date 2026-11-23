use std::fs;
use std::path::{Path, PathBuf};

use crate::archive::NewFormat;
use crate::columns::{ArchiveColumns, ColumnId, FsColumns, max_width, min_width};
use crate::fsops;
use crate::theme::ThemeId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Setting {
    SingleClickOpen,
    ConfirmDelete,
    ShowHidden,
    OpenAfterExtract,
    AutoPreview,
    ConfirmMove,
    AutoNfoFull,
    NfoColors,
    HexColors,
    CodeWrap,
    BackupEdits,
    RestoreSession,
    AudioAutoplay,
}

impl Setting {
    pub(crate) const ALL: [Setting; 13] = [
        Setting::OpenAfterExtract,
        Setting::SingleClickOpen,
        Setting::AutoPreview,
        Setting::ConfirmDelete,
        Setting::ConfirmMove,
        Setting::ShowHidden,
        Setting::AutoNfoFull,
        Setting::NfoColors,
        Setting::HexColors,
        Setting::CodeWrap,
        Setting::BackupEdits,
        Setting::RestoreSession,
        Setting::AudioAutoplay,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Setting::SingleClickOpen => "Single click opens folders and archives",
            Setting::ConfirmDelete => "Confirm before deleting",
            Setting::ShowHidden => "Show hidden files",
            Setting::OpenAfterExtract => "Open destination after extraction",
            Setting::AutoPreview => "Preview files when selected",
            Setting::ConfirmMove => "Confirm cross-pane moves",
            Setting::AutoNfoFull => "Open root NFO in full view",
            Setting::NfoColors => "Use ANSI colors in NFO previews",
            Setting::HexColors => "Use byte colors in hex previews",
            Setting::CodeWrap => "Wrap code previews by default",
            Setting::BackupEdits => "Keep a .bak before archive edits",
            Setting::RestoreSession => "Restore tabs on startup",
            Setting::AudioAutoplay => "Autoplay audio previews",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Setting::SingleClickOpen => {
                "Use Ctrl or Shift to select without opening. Double click always opens."
            }
            Setting::ConfirmDelete => "Ask before removing entries from an archive.",
            Setting::ShowHidden => "Include dotfiles in the files pane.",
            Setting::OpenAfterExtract => {
                "Navigate to the extracted files and highlight them in Files."
            }
            Setting::AutoPreview => {
                "Show files up to 64 MiB in the preview pane on click. Audio uses a 1 GiB limit."
            }
            Setting::ConfirmMove => "Ask before deleting sources during a move.",
            Setting::AutoNfoFull => "Open the full NFO viewer when an archive contains a root NFO.",
            Setting::NfoColors => "Preserve ANSI colors and bold attributes in NFO art.",
            Setting::HexColors => "Color zeroes, controls, printable bytes, and high bytes.",
            Setting::CodeWrap => "Disable horizontal scrolling for long code lines.",
            Setting::BackupEdits => "Copy the archive to .bak before replacing it.",
            Setting::RestoreSession => "Reopen tabs and archive paths on the next launch.",
            Setting::AudioAutoplay => "Start playback when an audio file is previewed.",
        }
    }

    pub(crate) fn value(self, settings: &Settings) -> bool {
        match self {
            Setting::SingleClickOpen => settings.single_click_open,
            Setting::ConfirmDelete => settings.confirm_delete,
            Setting::ShowHidden => settings.show_hidden,
            Setting::OpenAfterExtract => settings.open_after_extract,
            Setting::AutoPreview => settings.auto_preview,
            Setting::ConfirmMove => settings.confirm_move,
            Setting::AutoNfoFull => settings.auto_nfo_full,
            Setting::NfoColors => settings.nfo_colors,
            Setting::HexColors => settings.hex_colors,
            Setting::CodeWrap => settings.code_wrap,
            Setting::BackupEdits => settings.backup_edits,
            Setting::RestoreSession => settings.restore_session,
            Setting::AudioAutoplay => settings.audio_autoplay,
        }
    }

    pub(crate) fn set(self, settings: &mut Settings, value: bool) {
        match self {
            Setting::SingleClickOpen => settings.single_click_open = value,
            Setting::ConfirmDelete => settings.confirm_delete = value,
            Setting::ShowHidden => settings.show_hidden = value,
            Setting::OpenAfterExtract => settings.open_after_extract = value,
            Setting::AutoPreview => settings.auto_preview = value,
            Setting::ConfirmMove => settings.confirm_move = value,
            Setting::AutoNfoFull => settings.auto_nfo_full = value,
            Setting::NfoColors => settings.nfo_colors = value,
            Setting::HexColors => settings.hex_colors = value,
            Setting::CodeWrap => settings.code_wrap = value,
            Setting::BackupEdits => settings.backup_edits = value,
            Setting::RestoreSession => settings.restore_session = value,
            Setting::AudioAutoplay => settings.audio_autoplay = value,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Settings {
    pub(crate) single_click_open: bool,
    pub(crate) confirm_delete: bool,
    pub(crate) show_hidden: bool,
    pub(crate) open_after_extract: bool,
    pub(crate) auto_preview: bool,
    pub(crate) confirm_move: bool,
    pub(crate) auto_nfo_full: bool,
    pub(crate) nfo_colors: bool,
    pub(crate) hex_colors: bool,
    pub(crate) code_wrap: bool,
    pub(crate) backup_edits: bool,
    pub(crate) restore_session: bool,
    pub(crate) audio_autoplay: bool,
    pub(crate) nfo_font_size: f32,
    pub(crate) edit_compression: u8,
    pub(crate) default_format: NewFormat,
    pub(crate) default_compression: u8,
    pub(crate) theme: ThemeId,
    pub(crate) split: f32,
    pub(crate) preview_ratio: f32,
    pub(crate) fs_columns: FsColumns,
    pub(crate) archive_columns: ArchiveColumns,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            single_click_open: true,
            confirm_delete: true,
            show_hidden: false,
            open_after_extract: true,
            auto_preview: true,
            confirm_move: true,
            auto_nfo_full: true,
            nfo_colors: true,
            hex_colors: true,
            code_wrap: false,
            backup_edits: true,
            restore_session: true,
            audio_autoplay: false,
            nfo_font_size: 13.0,
            edit_compression: 5,
            default_format: NewFormat::SevenZ,
            default_compression: 5,
            theme: ThemeId::Dark,
            split: 0.5,
            preview_ratio: 0.32,
            fs_columns: FsColumns::default(),
            archive_columns: ArchiveColumns::default(),
        }
    }
}

impl Settings {
    pub(crate) fn load() -> Self {
        Self::load_from(&Self::path())
    }

    pub(crate) fn load_from(path: &Path) -> Self {
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        let mut settings = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let raw = value.trim();
            let enabled = raw.eq_ignore_ascii_case("true");
            match key.trim() {
                "single_click_open" => settings.single_click_open = enabled,
                "confirm_delete" => settings.confirm_delete = enabled,
                "show_hidden" => settings.show_hidden = enabled,
                "open_after_extract" => settings.open_after_extract = enabled,
                "auto_preview" => settings.auto_preview = enabled,
                "confirm_move" => settings.confirm_move = enabled,
                "auto_nfo_full" => settings.auto_nfo_full = enabled,
                "nfo_colors" => settings.nfo_colors = enabled,
                "hex_colors" => settings.hex_colors = enabled,
                "code_wrap" => settings.code_wrap = enabled,
                "backup_edits" => settings.backup_edits = enabled,
                "restore_session" => settings.restore_session = enabled,
                "audio_autoplay" => settings.audio_autoplay = enabled,
                "nfo_font_size" => {
                    if let Ok(value) = raw.parse::<f32>()
                        && value.is_finite()
                    {
                        settings.nfo_font_size = value.clamp(10.0, 22.0);
                    }
                }
                "edit_compression" => {
                    if let Ok(value) = raw.parse::<u8>() {
                        settings.edit_compression = value.min(9);
                    }
                }
                "default_compression" => {
                    if let Ok(value) = raw.parse::<u8>() {
                        settings.default_compression = value.min(9);
                    }
                }
                "default_format" => {
                    if let Some(format) = NewFormat::ALL
                        .iter()
                        .find(|format| format.label() == raw)
                        .copied()
                    {
                        settings.default_format = format;
                    }
                }
                "theme" => {
                    if let Some(theme) = ThemeId::parse(raw) {
                        settings.theme = theme;
                    }
                }
                "split" => {
                    if let Ok(value) = raw.parse::<f32>()
                        && value.is_finite()
                    {
                        settings.split = value.clamp(0.15, 0.85);
                    }
                }
                "preview_ratio" => {
                    if let Ok(value) = raw.parse::<f32>()
                        && value.is_finite()
                    {
                        settings.preview_ratio = value.clamp(0.15, 0.75);
                    }
                }
                "fs_name" => {
                    settings.fs_columns.name = parse_optional_width(raw, ColumnId::Name);
                }
                "archive_name" => {
                    settings.archive_columns.name = parse_optional_width(raw, ColumnId::Name);
                }
                "fs_size" => {
                    if let Some(width) = parse_width(raw, ColumnId::Size) {
                        settings.fs_columns.set(ColumnId::Size, width);
                    }
                }
                "fs_modified" => {
                    if let Some(width) = parse_width(raw, ColumnId::Modified) {
                        settings.fs_columns.set(ColumnId::Modified, width);
                    }
                }
                "fs_kind" => {
                    if let Some(width) = parse_width(raw, ColumnId::Kind) {
                        settings.fs_columns.set(ColumnId::Kind, width);
                    }
                }
                "archive_size" => {
                    if let Some(width) = parse_width(raw, ColumnId::Size) {
                        settings.archive_columns.set(ColumnId::Size, width);
                    }
                }
                "archive_packed" => {
                    if let Some(width) = parse_width(raw, ColumnId::Packed) {
                        settings.archive_columns.set(ColumnId::Packed, width);
                    }
                }
                "archive_modified" => {
                    if let Some(width) = parse_width(raw, ColumnId::Modified) {
                        settings.archive_columns.set(ColumnId::Modified, width);
                    }
                }
                "archive_method" => {
                    if let Some(width) = parse_width(raw, ColumnId::Kind) {
                        settings.archive_columns.set(ColumnId::Kind, width);
                    }
                }
                _ => {}
            }
        }
        settings
    }

    pub(crate) fn save(&self) -> Result<(), String> {
        self.save_to(&Self::path())
    }

    pub(crate) fn save_to(&self, path: &Path) -> Result<(), String> {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| "Invalid settings path".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
        let contents = format!(
            "single_click_open={}\nconfirm_delete={}\nshow_hidden={}\nopen_after_extract={}\nauto_preview={}\nconfirm_move={}\nauto_nfo_full={}\nnfo_colors={}\nhex_colors={}\ncode_wrap={}\nbackup_edits={}\nrestore_session={}\naudio_autoplay={}\nnfo_font_size={:.2}\nedit_compression={}\ndefault_format={}\ndefault_compression={}\ntheme={}\nsplit={:.4}\npreview_ratio={:.4}\nfs_name={:.1}\nfs_size={:.1}\nfs_modified={:.1}\nfs_kind={:.1}\narchive_name={:.1}\narchive_size={:.1}\narchive_packed={:.1}\narchive_modified={:.1}\narchive_method={:.1}\n",
            self.single_click_open,
            self.confirm_delete,
            self.show_hidden,
            self.open_after_extract,
            self.auto_preview,
            self.confirm_move,
            self.auto_nfo_full,
            self.nfo_colors,
            self.hex_colors,
            self.code_wrap,
            self.backup_edits,
            self.restore_session,
            self.audio_autoplay,
            self.nfo_font_size,
            self.edit_compression,
            self.default_format.label(),
            self.default_compression,
            self.theme.key(),
            self.split,
            self.preview_ratio,
            self.fs_columns.name.unwrap_or(0.0),
            self.fs_columns.size,
            self.fs_columns.modified,
            self.fs_columns.kind,
            self.archive_columns.name.unwrap_or(0.0),
            self.archive_columns.size,
            self.archive_columns.packed,
            self.archive_columns.modified,
            self.archive_columns.method
        );
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, contents)
            .map_err(|error| format!("Could not write {}: {error}", temporary.display()))?;
        fs::rename(&temporary, path)
            .map_err(|error| format!("Could not save {}: {error}", path.display()))
    }

    pub(crate) fn path() -> PathBuf {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| fsops::home_dir().join(".config"));
        base.join("7zrn").join("settings.conf")
    }
}
fn parse_width(value: &str, column: ColumnId) -> Option<f32> {
    let width = value.trim().parse::<f32>().ok()?;
    if !width.is_finite() {
        return None;
    }
    Some(width.clamp(min_width(column), max_width(column)))
}

fn parse_optional_width(value: &str, column: ColumnId) -> Option<f32> {
    let width = value.trim().parse::<f32>().ok()?;
    if !width.is_finite() || width <= 0.0 {
        return None;
    }
    Some(width.clamp(min_width(column), max_width(column)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn settings_round_trip() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("7zrn-settings-{}-{stamp}", std::process::id()));
        let path = directory.join("settings.conf");
        let settings = Settings {
            single_click_open: true,
            confirm_delete: false,
            show_hidden: true,
            open_after_extract: false,
            auto_preview: false,
            confirm_move: false,
            auto_nfo_full: false,
            nfo_colors: false,
            hex_colors: false,
            code_wrap: true,
            backup_edits: false,
            restore_session: false,
            audio_autoplay: true,
            nfo_font_size: 15.0,
            edit_compression: 7,
            default_format: NewFormat::Zip,
            default_compression: 3,
            theme: ThemeId::Nord,
            split: 0.37,
            preview_ratio: 0.41,
            fs_columns: FsColumns {
                name: Some(222.0),
                size: 101.0,
                modified: 151.0,
                kind: 99.0,
            },
            archive_columns: ArchiveColumns {
                name: Some(180.0),
                size: 91.0,
                packed: 81.0,
                modified: 141.0,
                method: 95.0,
            },
        };
        settings.save_to(&path).unwrap();
        let loaded = Settings::load_from(&path);
        assert!(loaded.single_click_open);
        assert!(!loaded.confirm_delete);
        assert!(loaded.show_hidden);
        assert!(!loaded.open_after_extract);
        assert!(!loaded.auto_preview);
        assert!(!loaded.confirm_move);
        assert!(!loaded.auto_nfo_full);
        assert!(!loaded.nfo_colors);
        assert!(!loaded.hex_colors);
        assert!(loaded.code_wrap);
        assert!(!loaded.backup_edits);
        assert!(!loaded.restore_session);
        assert!(loaded.audio_autoplay);
        assert_eq!(loaded.nfo_font_size, 15.0);
        assert_eq!(loaded.edit_compression, 7);
        assert_eq!(loaded.default_format, NewFormat::Zip);
        assert_eq!(loaded.default_compression, 3);
        assert_eq!(loaded.theme, ThemeId::Nord);
        assert!((loaded.split - 0.37).abs() < 0.001);
        assert!((loaded.preview_ratio - 0.41).abs() < 0.001);
        assert_eq!(loaded.fs_columns.name, Some(222.0));
        assert_eq!(loaded.fs_columns.size, 101.0);
        assert_eq!(loaded.fs_columns.modified, 151.0);
        assert_eq!(loaded.fs_columns.kind, 99.0);
        assert_eq!(loaded.archive_columns.name, Some(180.0));
        assert_eq!(loaded.archive_columns.size, 91.0);
        assert_eq!(loaded.archive_columns.packed, 81.0);
        assert_eq!(loaded.archive_columns.modified, 141.0);
        assert_eq!(loaded.archive_columns.method, 95.0);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn missing_settings_use_defaults() {
        let path = std::env::temp_dir().join("7zrn-settings-that-do-not-exist");
        let settings = Settings::load_from(&path);
        assert!(settings.single_click_open);
        assert!(settings.confirm_delete);
        assert!(!settings.show_hidden);
        assert!(settings.open_after_extract);
        assert!(settings.auto_preview);
        assert_eq!(settings.theme, ThemeId::Dark);
        assert!((settings.split - 0.5).abs() < 0.001);
    }
}
