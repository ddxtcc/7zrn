#[derive(Clone, Debug)]
pub(crate) struct HexPreview {
    pub(crate) text: String,
}

pub(crate) fn format(bytes: &[u8], limit: usize) -> HexPreview {
    let truncated = bytes.len() > limit;
    let bytes = &bytes[..bytes.len().min(limit)];
    let mut text = String::new();
    for (line, chunk) in bytes.chunks(16).enumerate() {
        if line > 0 {
            text.push('\n');
        }
        text.push_str(&format!("{:08x}  ", line * 16));
        for index in 0..16 {
            if let Some(byte) = chunk.get(index) {
                text.push_str(&format!("{byte:02x} "));
            } else {
                text.push_str("   ");
            }
            if index == 7 {
                text.push(' ');
            }
        }
        text.push_str(" |");
        for byte in chunk {
            text.push(if (0x20..=0x7e).contains(byte) {
                *byte as char
            } else {
                '.'
            });
        }
        for _ in chunk.len()..16 {
            text.push(' ');
        }
        text.push('|');
    }
    if truncated {
        text.push_str("\n[hex preview truncated]");
    }
    HexPreview { text }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_offsets_bytes_and_ascii() {
        let preview = format(b"hello\0world", 1024);
        assert!(
            preview
                .text
                .starts_with("00000000  68 65 6c 6c 6f 00 77 6f  72 6c 64")
        );
        assert!(preview.text.contains("|hello.world"));
        assert!(!preview.text.contains("truncated"));
    }

    #[test]
    fn marks_truncation() {
        let preview = format(&[0u8; 32], 16);
        assert!(preview.text.contains("truncated"));
        assert!(preview.text.ends_with("[hex preview truncated]"));
    }
}
