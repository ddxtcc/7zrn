use std::fs;
use std::path::Path;

use gpui::{Context, Window};

use crate::archive;
use crate::state::{Modal, SearchHit, SearchScope};

use super::AppView;

const MAX_RESULTS: usize = 500;
const MAX_VISITED: usize = 20_000;
const MAX_CONTENT: u64 = 1024 * 1024;

impl AppView {
    pub(crate) fn open_search_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let scope = if self.focused == crate::state::Pane::Archive && self.archive.archive.is_some()
        {
            SearchScope::Archive
        } else {
            SearchScope::Fs
        };
        self.modal = Some(Modal::Search {
            scope,
            query: String::new(),
            content: false,
            results: Vec::new(),
            selected: 0,
            error: None,
        });
        self.begin_edit(crate::state::EditTarget::Search, String::new(), window, cx);
    }

    pub(crate) fn set_search_scope(&mut self, scope: SearchScope, cx: &mut Context<Self>) {
        if let Some(Modal::Search {
            scope: current,
            results,
            selected,
            error,
            ..
        }) = &mut self.modal
            && *current != scope
        {
            *current = scope;
            results.clear();
            *selected = 0;
            *error = None;
        }
        self.run_search(cx);
    }

    pub(crate) fn toggle_search_content(&mut self, cx: &mut Context<Self>) {
        if let Some(Modal::Search {
            content,
            results,
            selected,
            error,
            ..
        }) = &mut self.modal
        {
            *content = !*content;
            results.clear();
            *selected = 0;
            *error = None;
        }
        self.run_search(cx);
    }

    pub(crate) fn run_search(&mut self, cx: &mut Context<Self>) {
        let (scope, query, content) = match &self.modal {
            Some(Modal::Search {
                scope,
                query,
                content,
                ..
            }) => (*scope, query.trim().to_string(), *content),
            _ => return,
        };
        if query.is_empty() {
            if let Some(Modal::Search {
                results,
                selected,
                error,
                ..
            }) = &mut self.modal
            {
                results.clear();
                *selected = 0;
                *error = None;
            }
            cx.notify();
            return;
        }
        let needle = query.to_lowercase();
        let result = match scope {
            SearchScope::Fs => search_fs(&self.fs.path, &needle, content, self.fs.show_hidden),
            SearchScope::Archive => {
                if content {
                    Err("Content search is available in the Files pane.".to_string())
                } else {
                    Ok(search_archive(&self.archive, &needle))
                }
            }
        };
        if let Some(Modal::Search {
            results,
            selected,
            error,
            ..
        }) = &mut self.modal
        {
            match result {
                Ok(hits) => {
                    *results = hits;
                    *selected = 0;
                    *error = None;
                }
                Err(message) => {
                    results.clear();
                    *selected = 0;
                    *error = Some(message);
                }
            }
        }
        cx.notify();
    }

    pub(crate) fn activate_search_hit(&mut self, hit: SearchHit, cx: &mut Context<Self>) {
        self.modal = None;
        self.edit = None;
        match hit {
            SearchHit::Fs { path, is_dir, .. } => {
                self.focused = crate::state::Pane::Fs;
                if is_dir {
                    self.navigate_fs(path, true, cx);
                } else {
                    let parent = path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| self.fs.path.clone());
                    self.navigate_fs(parent, true, cx);
                    self.select_fs_path(&path, cx);
                }
            }
            SearchHit::Archive { key, is_dir, .. } => {
                self.archive.inner = if is_dir {
                    key.clone()
                } else {
                    archive::parent_inner(&key)
                };
                self.archive.filter.clear();
                self.focused = crate::state::Pane::Archive;
                self.archive.active = None;
                self.archive.anchor = None;
                self.archive.selected.clear();
                if !is_dir {
                    let children = self.arc_visible_children();
                    if let Some(index) = children
                        .iter()
                        .position(|child| self.archive.key_for(&child.name) == key)
                    {
                        self.archive.active = Some(index);
                        self.archive.anchor = Some(index);
                        self.archive.selected.insert(key);
                    }
                }
                self.archive.reset_active();
                cx.notify();
            }
        }
    }

    pub(crate) fn move_search_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        if let Some(Modal::Search {
            results, selected, ..
        }) = &mut self.modal
            && !results.is_empty()
        {
            let len = results.len() as isize;
            *selected = (*selected as isize + delta).clamp(0, len - 1) as usize;
        }
        cx.notify();
    }

    pub(crate) fn submit_search(&mut self, cx: &mut Context<Self>) {
        let hit = match &self.modal {
            Some(Modal::Search {
                results, selected, ..
            }) => results.get(*selected).cloned(),
            _ => None,
        };
        if let Some(hit) = hit {
            self.activate_search_hit(hit, cx);
        } else {
            self.run_search(cx);
        }
    }
}

fn search_fs(
    root: &Path,
    needle: &str,
    content: bool,
    show_hidden: bool,
) -> Result<Vec<SearchHit>, String> {
    let mut hits = Vec::new();
    let mut visited = 0;
    walk_search(
        root,
        root,
        needle,
        content,
        show_hidden,
        &mut visited,
        &mut hits,
    );
    hits.sort_by(|left, right| {
        left.label()
            .to_lowercase()
            .cmp(&right.label().to_lowercase())
    });
    Ok(hits)
}

fn walk_search(
    root: &Path,
    directory: &Path,
    needle: &str,
    content: bool,
    show_hidden: bool,
    visited: &mut usize,
    hits: &mut Vec<SearchHit>,
) {
    if *visited >= MAX_VISITED || hits.len() >= MAX_RESULTS {
        return;
    }
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries {
        if *visited >= MAX_VISITED || hits.len() >= MAX_RESULTS {
            return;
        }
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        *visited += 1;
        let is_dir = meta.is_dir();
        let relative = path
            .strip_prefix(root)
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| name.clone());
        let name_match = name.to_lowercase().contains(needle);
        let content_match = content
            && !is_dir
            && meta.is_file()
            && meta.len() <= MAX_CONTENT
            && fs::read(&path)
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .is_some_and(|text| text.to_lowercase().contains(needle));
        if name_match || content_match {
            hits.push(SearchHit::Fs {
                path: path.clone(),
                label: if content_match && !name_match {
                    format!("{relative} (content match)")
                } else {
                    relative
                },
                is_dir,
            });
        }
        if is_dir && !meta.file_type().is_symlink() {
            walk_search(root, &path, needle, content, show_hidden, visited, hits);
        }
    }
}

fn search_archive(archive: &crate::state::ArchiveState, needle: &str) -> Vec<SearchHit> {
    archive
        .listing
        .as_ref()
        .map(|listing| {
            listing
                .entries
                .iter()
                .filter(|entry| entry.display_path.to_lowercase().contains(needle))
                .take(MAX_RESULTS)
                .map(|entry| SearchHit::Archive {
                    key: entry.display_path.clone(),
                    label: entry.display_path.clone(),
                    is_dir: entry.is_dir,
                })
                .collect()
        })
        .unwrap_or_default()
}

impl SearchHit {
    fn label(&self) -> &str {
        match self {
            SearchHit::Fs { label, .. } | SearchHit::Archive { label, .. } => label,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn recursively_searches_names_and_contents() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("7zrn-search-{}-{stamp}", std::process::id()));
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("needle.txt"), "no match").unwrap();
        fs::write(root.join("sub/inside.txt"), "the needle is here").unwrap();
        fs::write(root.join(".hidden.txt"), "needle").unwrap();

        let names = search_fs(&root, "needle", false, false).unwrap();
        assert_eq!(names.len(), 1);
        let content = search_fs(&root, "needle", true, false).unwrap();
        assert_eq!(content.len(), 2);
        assert!(
            content
                .iter()
                .any(|hit| hit.label().contains("content match"))
        );
        fs::remove_dir_all(root).unwrap();
    }
}
