#[derive(Clone, Debug)]
pub(crate) struct HexPreview {
    pub(crate) lines: Vec<HexLine>,
    pub(crate) truncated: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct HexLine {
    pub(crate) offset: usize,
    pub(crate) bytes: Vec<u8>,
}

pub(crate) fn header() -> String {
    let mut text = format!("{:>8}  ", "OFFSET");
    for index in 0..16 {
        text.push_str(&format!("{index:02X} "));
        if index == 7 {
            text.push(' ');
        }
    }
    text.push_str(" |ASCII");
    for _ in 5..16 {
        text.push(' ');
    }
    text.push('|');
    text
}

pub(crate) fn format(bytes: &[u8], limit: usize) -> HexPreview {
    let truncated = bytes.len() > limit;
    let bytes = &bytes[..bytes.len().min(limit)];
    let lines = bytes
        .chunks(16)
        .enumerate()
        .map(|(line, chunk)| HexLine {
            offset: line * 16,
            bytes: chunk.to_vec(),
        })
        .collect();
    HexPreview { lines, truncated }
}

impl HexPreview {
    pub(crate) fn text(&self) -> String {
        let mut text = header();
        text.push('\n');
        for (index, line) in self.lines.iter().enumerate() {
            if index > 0 {
                text.push('\n');
            }
            push_line(&mut text, line);
        }
        if self.truncated {
            text.push_str("\n[hex preview truncated]");
        }
        text
    }
}

pub(crate) fn push_line(text: &mut String, line: &HexLine) {
    text.push_str(&format!("{:08x}  ", line.offset));
    for index in 0..16 {
        if let Some(byte) = line.bytes.get(index) {
            text.push_str(&format!("{byte:02x} "));
        } else {
            text.push_str("   ");
        }
        if index == 7 {
            text.push(' ');
        }
    }
    text.push_str(" |");
    for byte in &line.bytes {
        text.push(if (0x20..=0x7e).contains(byte) {
            *byte as char
        } else {
            '.'
        });
    }
    for _ in line.bytes.len()..16 {
        text.push(' ');
    }
    text.push('|');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_offsets_bytes_and_ascii() {
        let preview = format(b"hello\0world", 1024);
        let text = preview.text();
        assert!(text.chars().count() > 0);
        assert!(text.contains("00000000  68 65 6c 6c 6f 00 77 6f  72 6c 64"));
        assert!(text.contains("|hello.world"));
        assert!(!text.contains("truncated"));
        assert!(text.starts_with("  OFFSET"));
    }

    #[test]
    fn marks_truncation() {
        let preview = format(&[0u8; 32], 16);
        let text = preview.text();
        assert!(text.contains("truncated"));
        assert!(text.ends_with("[hex preview truncated]"));
    }
}
