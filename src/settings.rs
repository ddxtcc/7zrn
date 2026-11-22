use std::fs;
use std::path::{Path, PathBuf};

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
}

impl Setting {
    pub(crate) const ALL: [Setting; 5] = [
        Setting::OpenAfterExtract,
        Setting::SingleClickOpen,
        Setting::AutoPreview,
        Setting::ConfirmDelete,
        Setting::ShowHidden,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Setting::SingleClickOpen => "Single click opens folders and archives",
            Setting::ConfirmDelete => "Confirm before deleting",
            Setting::ShowHidden => "Show hidden files",
            Setting::OpenAfterExtract => "Open destination after extraction",
            Setting::AutoPreview => "Preview files when selected",
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
            Setting::AutoPreview => "Show files up to 4 MiB in the preview pane on click.",
        }
    }

    pub(crate) fn value(self, settings: &Settings) -> bool {
        match self {
            Setting::SingleClickOpen => settings.single_click_open,
            Setting::ConfirmDelete => settings.confirm_delete,
            Setting::ShowHidden => settings.show_hidden,
            Setting::OpenAfterExtract => settings.open_after_extract,
            Setting::AutoPreview => settings.auto_preview,
        }
    }

    pub(crate) fn set(self, settings: &mut Settings, value: bool) {
        match self {
            Setting::SingleClickOpen => settings.single_click_open = value,
            Setting::ConfirmDelete => settings.confirm_delete = value,
            Setting::ShowHidden => settings.show_hidden = value,
            Setting::OpenAfterExtract => settings.open_after_extract = value,
            Setting::AutoPreview => settings.auto_preview = value,
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
            "single_click_open={}\nconfirm_delete={}\nshow_hidden={}\nopen_after_extract={}\nauto_preview={}\ntheme={}\nsplit={:.4}\npreview_ratio={:.4}\nfs_name={:.1}\nfs_size={:.1}\nfs_modified={:.1}\nfs_kind={:.1}\narchive_name={:.1}\narchive_size={:.1}\narchive_packed={:.1}\narchive_modified={:.1}\narchive_method={:.1}\n",
            self.single_click_open,
            self.confirm_delete,
            self.show_hidden,
            self.open_after_extract,
            self.auto_preview,
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
