use std::fs;
use std::path::{Path, PathBuf};

use crate::settings::Settings;

#[derive(Clone, Debug, Default)]
pub(crate) struct Session {
    pub(crate) active: usize,
    pub(crate) tabs: Vec<SessionTab>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SessionTab {
    pub(crate) fs: String,
    pub(crate) archive: Option<String>,
}

impl Session {
    pub(crate) fn load() -> Self {
        Self::load_from(&Self::path())
    }

    pub(crate) fn load_from(path: &Path) -> Self {
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        let mut session = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "active" => {
                    if let Ok(active) = value.trim().parse::<usize>() {
                        session.active = active;
                    }
                }
                "tab" => {
                    let (fs_path, archive) = value.split_once('\t').unwrap_or((value, ""));
                    let archive = (!archive.trim().is_empty()).then(|| archive.trim().to_string());
                    session.tabs.push(SessionTab {
                        fs: fs_path.trim().to_string(),
                        archive,
                    });
                }
                _ => {}
            }
        }
        session
    }

    pub(crate) fn save(&self) -> Result<(), String> {
        self.save_to(&Self::path())
    }

    pub(crate) fn save_to(&self, path: &Path) -> Result<(), String> {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| "Invalid session path".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
        let mut contents = format!("active={}\n", self.active);
        for tab in &self.tabs {
            contents.push_str("tab=");
            contents.push_str(&tab.fs.replace('\t', " "));
            contents.push('\t');
            if let Some(archive) = &tab.archive {
                contents.push_str(archive);
            }
            contents.push('\n');
        }
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, contents)
            .map_err(|error| format!("Could not write {}: {error}", temporary.display()))?;
        fs::rename(&temporary, path)
            .map_err(|error| format!("Could not save {}: {error}", path.display()))
    }

    pub(crate) fn path() -> PathBuf {
        Settings::path().with_file_name("session.conf")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn session_round_trip() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("7zrn-session-{}-{stamp}", std::process::id()));
        let path = directory.join("session.conf");
        let session = Session {
            active: 1,
            tabs: vec![
                SessionTab {
                    fs: "/tmp".into(),
                    archive: None,
                },
                SessionTab {
                    fs: "/home".into(),
                    archive: Some("/tmp/a.zip".into()),
                },
            ],
        };
        session.save_to(&path).unwrap();
        let loaded = Session::load_from(&path);
        assert_eq!(loaded.active, 1);
        assert_eq!(loaded.tabs.len(), 2);
        assert_eq!(loaded.tabs[1].archive.as_deref(), Some("/tmp/a.zip"));
        assert_eq!(loaded.tabs[0].fs, "/tmp");
        let _ = fs::remove_dir_all(directory);
    }
}
