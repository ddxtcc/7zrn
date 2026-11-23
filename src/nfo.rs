use std::ops::Range;

#[derive(Clone, Debug)]
pub(crate) struct NfoDocument {
    pub(crate) text: String,
    pub(crate) colors: Vec<NfoColorSpan>,
}

#[derive(Clone, Debug)]
pub(crate) struct NfoColorSpan {
    pub(crate) range: Range<usize>,
    pub(crate) color: u8,
    pub(crate) bold: bool,
}

pub(crate) fn decode(bytes: &[u8]) -> NfoDocument {
    let source = match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => decode_cp437(bytes),
    };
    parse_ansi(&source)
}

pub(crate) fn nfo_color(color: u8) -> u32 {
    match color {
        0 => 0x121317,
        1 => 0xd65b5b,
        2 => 0x78c46d,
        3 => 0xd6c05b,
        4 => 0x6d8fd6,
        5 => 0xc06dd6,
        6 => 0x5bc4c4,
        7 => 0xd0d3da,
        8 => 0x5a5e68,
        9 => 0xff7b7b,
        10 => 0x9eed8a,
        11 => 0xffee84,
        12 => 0x8fb4ff,
        13 => 0xee8fff,
        14 => 0x7ff2ef,
        _ => 0xffffff,
    }
}

fn parse_ansi(source: &str) -> NfoDocument {
    let mut text = String::new();
    let mut colors = Vec::new();
    let mut color = 7u8;
    let mut bold = false;
    let mut chars = source.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\x1b' {
            if chars.peek() == Some(&'[') {
                let _ = chars.next();
                let mut sequence = String::new();
                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() {
                        if next == 'm' {
                            apply_sgr(&sequence, &mut color, &mut bold);
                        }
                        break;
                    }
                    sequence.push(next);
                }
            }
            continue;
        }
        match character {
            '\r' => {}
            '\t' => {
                for _ in 0..4 {
                    append_char(&mut text, &mut colors, ' ', color, bold);
                }
            }
            '\n' => append_char(&mut text, &mut colors, '\n', color, bold),
            character if character.is_control() => {}
            character => append_char(&mut text, &mut colors, character, color, bold),
        }
    }
    NfoDocument { text, colors }
}

fn append_char(
    text: &mut String,
    colors: &mut Vec<NfoColorSpan>,
    character: char,
    color: u8,
    bold: bool,
) {
    let start = text.len();
    text.push(character);
    if color == 7 && !bold {
        return;
    }
    if let Some(last) = colors.last_mut()
        && last.range.end == start
        && last.color == color
        && last.bold == bold
    {
        last.range.end = text.len();
        return;
    }
    colors.push(NfoColorSpan {
        range: start..text.len(),
        color,
        bold,
    });
}

fn apply_sgr(sequence: &str, color: &mut u8, bold: &mut bool) {
    for value in sequence.split(';') {
        let code = value.parse::<u16>().unwrap_or(0);
        match code {
            0 => {
                *color = 7;
                *bold = false;
            }
            1 => *bold = true,
            22 => *bold = false,
            30..=37 => *color = (code - 30) as u8,
            39 => *color = 7,
            90..=97 => *color = (code - 90 + 8) as u8,
            _ => {}
        }
    }
}

fn decode_cp437(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| {
            if *byte < 0x80 {
                *byte as char
            } else {
                CP437[(byte - 0x80) as usize]
            }
        })
        .collect()
}

const CP437: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å', 'É', 'æ', 'Æ',
    'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', '¢', '£', '¥', '₧', 'ƒ', 'á', 'í', 'ó', 'ú', 'ñ', 'Ñ',
    'ª', 'º', '¿', '⌐', '¬', '½', '¼', '¡', '«', '»', '░', '▒', '▓', '│', '┤', '╡', '╢', '╖', '╕',
    '╣', '║', '╗', '╝', '╜', '╛', '┐', '└', '┴', '┬', '├', '─', '┼', '╞', '╟', '╚', '╔', '╩', '╦',
    '╠', '═', '╬', '╧', '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫', '╪', '┘', '┌', '█', '▄', '▌', '▐',
    '▀', 'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ', 'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩', '≡', '±',
    '≥', '≤', '⌠', '⌡', '÷', '≈', '°', '∙', '·', '√', 'ⁿ', '²', '■', '\u{00a0}',
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_cp437_boxes() {
        let document = decode(&[0xb0, 0xb1, 0xb2, 0xc4, 0xda]);
        assert_eq!(document.text, "░▒▓─┌");
        assert!(document.colors.is_empty());
    }

    #[test]
    fn strips_ansi_and_records_colors() {
        let document = decode(b"\x1b[1;32mgreen\x1b[0m normal");
        assert_eq!(document.text, "green normal");
        assert!(
            document
                .colors
                .iter()
                .any(|span| span.color == 2 && span.bold)
        );
    }

    #[test]
    fn preserves_utf8_art() {
        let document = decode("╔══╗\n║ok║\n╚══╝".as_bytes());
        assert_eq!(document.text, "╔══╗\n║ok║\n╚══╝");
    }
}
