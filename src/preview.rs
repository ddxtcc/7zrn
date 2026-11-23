use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::audio::{self, AudioPreview};
use crate::hex;
use crate::markdown::{self, Block};
use crate::nfo;

#[derive(Clone, Copy, Debug)]
pub(crate) struct PreviewLimits {
    pub(crate) text_bytes: usize,
    pub(crate) hex_bytes: usize,
    pub(crate) pdf_dpi: u32,
    pub(crate) audio_cover: bool,
}

impl Default for PreviewLimits {
    fn default() -> Self {
        Self {
            text_bytes: 256 * 1024,
            hex_bytes: 64 * 1024,
            pdf_dpi: 96,
            audio_cover: true,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PreviewContent {
    Loading,
    Text(String),
    Markdown { source: String, blocks: Vec<Block> },
    Nfo(nfo::NfoDocument),
    Hex(hex::HexPreview),
    Audio(AudioPreview),
    Image { path: PathBuf },
    Unsupported(String),
    Error(String),
}

impl PreviewContent {
    pub(crate) fn copy_text(&self) -> Option<String> {
        match self {
            PreviewContent::Text(text) => Some(text.clone()),
            PreviewContent::Markdown { source, .. } => Some(source.clone()),
            PreviewContent::Nfo(document) => Some(document.text.clone()),
            PreviewContent::Hex(preview) => Some(preview.text()),
            _ => None,
        }
    }
}

pub(crate) fn load(path: &Path, limits: PreviewLimits) -> PreviewContent {
    if !path.is_file() {
        return PreviewContent::Error("Not a file".into());
    }
    if is_image(path) {
        return PreviewContent::Image {
            path: path.to_path_buf(),
        };
    }
    if audio::is_audio(path) {
        return match audio::probe(path, limits.audio_cover) {
            Ok(info) => PreviewContent::Audio(info),
            Err(message) => PreviewContent::Unsupported(message),
        };
    }
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
    {
        return match render_pdf(path, limits.pdf_dpi) {
            Ok(path) => PreviewContent::Image { path },
            Err(message) => PreviewContent::Unsupported(message),
        };
    }
    let markdown = is_markdown(path);
    let nfo = is_nfo(path);
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => return PreviewContent::Error(format!("Could not read file: {error}")),
    };
    let mut bytes = Vec::new();
    if let Err(error) = file
        .take(limits.text_bytes as u64 + 1)
        .read_to_end(&mut bytes)
    {
        return PreviewContent::Error(format!("Could not read file: {error}"));
    }
    if nfo {
        return PreviewContent::Nfo(nfo::decode(&bytes));
    }
    if bytes.contains(&0) {
        return PreviewContent::Hex(hex::format(&bytes, limits.hex_bytes));
    }
    let truncated = bytes.len() > limits.text_bytes;
    if truncated {
        bytes.truncate(limits.text_bytes);
    }
    let text = String::from_utf8_lossy(&bytes).to_string();
    if text
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return PreviewContent::Hex(hex::format(&bytes, limits.hex_bytes));
    }
    if text.is_empty() {
        return PreviewContent::Text("(empty file)".into());
    }
    let text = if truncated {
        format!("{text}\n\n[preview truncated]")
    } else {
        text
    };
    if markdown {
        PreviewContent::Markdown {
            source: text.clone(),
            blocks: markdown::parse(&text),
        }
    } else {
        PreviewContent::Text(text)
    }
}

fn is_nfo(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if name == "file_id.diz" {
        return true;
    }
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(extension.to_ascii_lowercase().as_str(), "nfo" | "diz")
}

fn is_markdown(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "md" | "markdown" | "mdown" | "mkd"
    )
}

fn render_pdf(path: &Path, dpi: u32) -> Result<PathBuf, String> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let directory = std::env::temp_dir().join(format!("7zrn-pdf-{}-{stamp}", std::process::id()));
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create PDF preview directory: {error}"))?;
    let output = directory.join("page");
    let result = std::process::Command::new("pdftoppm")
        .args(["-png", "-f", "1", "-singlefile", "-r"])
        .arg(dpi.to_string())
        .arg(path)
        .arg(&output)
        .output();
    match result {
        Ok(result) if result.status.success() => {
            let rendered = output.with_extension("png");
            if rendered.is_file() {
                Ok(rendered)
            } else {
                Err("PDF preview produced no image.".into())
            }
        }
        Ok(result) => {
            let detail = String::from_utf8_lossy(&result.stderr);
            Err(format!(
                "Could not render PDF. Install poppler-utils (pdftoppm).\n{}",
                detail.trim()
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err("PDF preview requires pdftoppm from poppler-utils.".into())
        }
        Err(error) => Err(format!("Could not render PDF: {error}")),
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
        match load(&text_path, PreviewLimits::default()) {
            PreviewContent::Text(text) => assert!(text.contains("hello")),
            other => panic!("expected text preview, got {other:?}"),
        }
        std::fs::remove_file(&text_path).unwrap();

        let binary_path = temp_path("binary.bin");
        std::fs::write(&binary_path, [0, 1, 2, 3]).unwrap();
        assert!(matches!(
            load(&binary_path, PreviewLimits::default()),
            PreviewContent::Hex(_)
        ));
        std::fs::remove_file(&binary_path).unwrap();
    }

    #[test]
    fn previews_markdown_by_extension() {
        let path = temp_path("readme.md");
        std::fs::write(&path, b"# Title\n\nBody\n").unwrap();
        match load(&path, PreviewLimits::default()) {
            PreviewContent::Markdown { blocks, .. } => assert!(matches!(
                blocks.first(),
                Some(Block::Heading { level: 1, .. })
            )),
            other => panic!("expected markdown preview, got {other:?}"),
        }
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn previews_images_by_extension() {
        let path = temp_path("image.png");
        std::fs::write(&path, []).unwrap();
        assert!(matches!(
            load(&path, PreviewLimits::default()),
            PreviewContent::Image { .. }
        ));
        std::fs::remove_file(&path).unwrap();
    }
}
