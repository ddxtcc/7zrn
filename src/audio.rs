use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub(crate) struct AudioPreview {
    pub(crate) title: Option<String>,
    pub(crate) artist: Option<String>,
    pub(crate) album: Option<String>,
    pub(crate) genre: Option<String>,
    pub(crate) duration: Option<f64>,
    pub(crate) bitrate: Option<u64>,
    pub(crate) sample_rate: Option<u32>,
    pub(crate) channels: Option<u32>,
    pub(crate) codec: Option<String>,
    pub(crate) cover: Option<PathBuf>,
}

pub(crate) fn is_audio(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "mp3"
            | "flac"
            | "wav"
            | "ogg"
            | "oga"
            | "opus"
            | "m4a"
            | "aac"
            | "wma"
            | "aiff"
            | "aif"
            | "alac"
            | "ape"
            | "mid"
            | "midi"
            | "mka"
            | "weba"
    )
}

pub(crate) fn probe(path: &Path) -> Result<AudioPreview, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "format=duration,bit_rate:format_tags=title,artist,album,genre:stream=codec_name,sample_rate,channels",
            "-of",
            "default=noprint_wrappers=1",
        ])
        .arg(path)
        .output()
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => {
                "Audio preview requires ffprobe from ffmpeg.".to_string()
            }
            _ => format!("Could not run ffprobe: {error}"),
        })?;
    if !output.status.success() {
        return Err(format!(
            "Could not inspect audio: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut info = AudioPreview {
        title: None,
        artist: None,
        album: None,
        genre: None,
        duration: None,
        bitrate: None,
        sample_rate: None,
        channels: None,
        codec: None,
        cover: None,
    };
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        match key.trim() {
            "duration" => info.duration = value.parse().ok(),
            "bit_rate" => info.bitrate = value.parse().ok(),
            "TAG:title" | "title" => info.title = Some(value.to_string()),
            "TAG:artist" | "artist" => info.artist = Some(value.to_string()),
            "TAG:album" | "album" => info.album = Some(value.to_string()),
            "TAG:genre" | "genre" => info.genre = Some(value.to_string()),
            "codec_name" => info.codec = Some(value.to_string()),
            "sample_rate" => info.sample_rate = value.parse().ok(),
            "channels" => info.channels = value.parse().ok(),
            _ => {}
        }
    }
    if info.duration.is_none() && info.codec.is_none() {
        return Err("This file does not contain an audio stream.".into());
    }
    info.cover = cover_art(path);
    Ok(info)
}

fn cover_art(path: &Path) -> Option<PathBuf> {
    let probe = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()
        .ok()?;
    if !probe.status.success() || String::from_utf8_lossy(&probe.stdout).trim().is_empty() {
        return None;
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let cover = std::env::temp_dir().join(format!("7zrn-cover-{}-{stamp}.png", std::process::id()));
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-i"])
        .arg(path)
        .args(["-map", "0:v:0", "-frames:v", "1", "-f", "image2"])
        .arg(&cover)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;
    if status.success() && cover.is_file() {
        Some(cover)
    } else {
        None
    }
}

pub(crate) fn format_duration(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

pub(crate) fn format_bitrate(bits: u64) -> String {
    format!("{} kbps", bits / 1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_durations() {
        assert_eq!(format_duration(0.0), "0:00");
        assert_eq!(format_duration(65.0), "1:05");
        assert_eq!(format_duration(3665.0), "1:01:05");
    }

    #[test]
    fn detects_audio_extensions() {
        assert!(is_audio(Path::new("track.mp3")));
        assert!(is_audio(Path::new("track.FLAC")));
        assert!(!is_audio(Path::new("track.txt")));
    }
}
