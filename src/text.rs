use std::path::PathBuf;

pub(crate) fn prev_boundary(text: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(text.len());
    while cursor > 0 && !text.is_char_boundary(cursor) {
        cursor -= 1;
    }
    if cursor == 0 {
        0
    } else {
        text[..cursor]
            .char_indices()
            .last()
            .map(|(index, _)| index)
            .unwrap_or(0)
    }
}

pub(crate) fn next_boundary(text: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(text.len());
    while cursor < text.len() && !text.is_char_boundary(cursor) {
        cursor += 1;
    }
    text[cursor..]
        .chars()
        .next()
        .map(|ch| cursor + ch.len_utf8())
        .unwrap_or(text.len())
}

pub(crate) fn clamp_boundary(text: &str, cursor: usize) -> usize {
    let mut cursor = cursor.min(text.len());
    while cursor > 0 && !text.is_char_boundary(cursor) {
        cursor -= 1;
    }
    cursor
}

pub(crate) fn replace_archive_extension(path: &str, extension: &str) -> String {
    let path_buf = PathBuf::from(path);
    if path_buf.is_dir() || path.ends_with('/') {
        return path_buf
            .join(format!("archive.{extension}"))
            .display()
            .to_string();
    }
    let Some(file_name) = path_buf.file_name().and_then(|n| n.to_str()) else {
        return format!("archive.{extension}");
    };
    let lower = file_name.to_ascii_lowercase();
    const KNOWN: [&str; 17] = [
        ".tar.gz", ".tar.bz2", ".tar.xz", ".tar.zst", ".tgz", ".tbz2", ".tbz", ".txz", ".tzst",
        ".7z", ".zip", ".tar", ".rar", ".gz", ".bz2", ".xz", ".zst",
    ];
    let mut base = file_name.to_string();
    for suffix in KNOWN {
        if lower.ends_with(suffix) {
            base = file_name[..file_name.len() - suffix.len()].to_string();
            break;
        }
    }
    if base.is_empty() {
        base = "archive".to_string();
    }
    let new_name = format!("{base}.{extension}");
    path_buf
        .parent()
        .map(|parent| parent.join(&new_name))
        .unwrap_or_else(|| PathBuf::from(new_name))
        .display()
        .to_string()
}
