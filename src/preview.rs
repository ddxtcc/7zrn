use std::fs::File;
use std::io::Read;
use std::path::Path;

const MAX_TEXT: usize = 256 * 1024;

#[derive(Clone, Debug)]
pub(crate) enum PreviewContent {
    Loading,
    Text(String),
    Image,
    Unsupported(String),
    Error(String),
}

pub(crate) fn load(path: &Path) -> PreviewContent {
    if !path.is_file() {
        return PreviewContent::Error("Not a file".into());
    }
    if is_image(path) {
        return PreviewContent::Image;
    }
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => return PreviewContent::Error(format!("Could not read file: {error}")),
    };
    let mut bytes = Vec::new();
    if let Err(error) = file.take(MAX_TEXT as u64 + 1).read_to_end(&mut bytes) {
        return PreviewContent::Error(format!("Could not read file: {error}"));
    }
    if bytes.contains(&0) {
        return PreviewContent::Unsupported("This file appears to be binary.".into());
    }
    let truncated = bytes.len() > MAX_TEXT;
    if truncated {
        bytes.truncate(MAX_TEXT);
    }
    let text = String::from_utf8_lossy(&bytes).to_string();
    if text
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return PreviewContent::Unsupported("This file appears to be binary.".into());
    }
    if text.is_empty() {
        return PreviewContent::Text("(empty file)".into());
    }
    if truncated {
        PreviewContent::Text(format!("{text}\n\n[preview truncated]"))
    } else {
        PreviewContent::Text(text)
    }
}

fn is_image(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "avif"
            | "jpg"
            | "jpeg"
            | "png"
            | "gif"
            | "webp"
            | "tif"
            | "tiff"
            | "tga"
            | "dds"
            | "bmp"
            | "ico"
            | "hdr"
            | "exr"
            | "pbm"
            | "pam"
            | "ppm"
            | "pgm"
            | "ff"
            | "farbfeld"
            | "qoi"
            | "svg"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> std::path::PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "7zrn-preview-{}-{stamp}-{name}",
            std::process::id()
        ))
    }

    #[test]
    fn previews_text_and_rejects_binary() {
        let text_path = temp_path("text.txt");
        std::fs::write(&text_path, b"hello\nworld\n").unwrap();
        match load(&text_path) {
            PreviewContent::Text(text) => assert!(text.contains("hello")),
            other => panic!("expected text preview, got {other:?}"),
        }
        std::fs::remove_file(&text_path).unwrap();

        let binary_path = temp_path("binary.bin");
        std::fs::write(&binary_path, [0, 1, 2, 3]).unwrap();
        assert!(matches!(load(&binary_path), PreviewContent::Unsupported(_)));
        std::fs::remove_file(&binary_path).unwrap();
    }

    #[test]
    fn previews_images_by_extension() {
        let path = temp_path("image.png");
        std::fs::write(&path, []).unwrap();
        assert!(matches!(load(&path), PreviewContent::Image));
        std::fs::remove_file(&path).unwrap();
    }
}
