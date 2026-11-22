use std::ops::Range;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SyntaxClass {
    Keyword,
    String,
    Number,
    Comment,
    Key,
    Literal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Syntax {
    Json,
    Rust,
    Python,
    Shell,
    Toml,
    Yaml,
    Css,
    Html,
    Javascript,
    Plain,
}

impl Syntax {
    pub(crate) fn detect(path: &Path) -> Self {
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            return Syntax::Plain;
        };
        match extension.to_ascii_lowercase().as_str() {
            "json" => Syntax::Json,
            "rs" => Syntax::Rust,
            "py" => Syntax::Python,
            "sh" | "bash" | "zsh" | "fish" => Syntax::Shell,
            "toml" => Syntax::Toml,
            "yaml" | "yml" => Syntax::Yaml,
            "css" => Syntax::Css,
            "html" | "htm" | "xml" => Syntax::Html,
            "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx" => Syntax::Javascript,
            _ => Syntax::Plain,
        }
    }

    fn rules(self) -> Rules {
        match self {
            Syntax::Json => Rules {
                line_comments: &[],
                block_comment: None,
                quotes: b"\"",
                keywords: &[],
                literals: &["true", "false", "null"],
                key_separators: b":",
                json_keys: true,
                html: false,
            },
            Syntax::Rust => Rules {
                line_comments: &[b"\x2f\x2f"],
                block_comment: Some((b"/*", b"*/")),
                quotes: b"\"'",
                keywords: &[
                    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                    "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
                    "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self",
                    "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
                    "while",
                ],
                literals: &["false", "true", "None", "Some", "Ok", "Err"],
                key_separators: &[],
                json_keys: false,
                html: false,
            },
            Syntax::Python => Rules {
                line_comments: &[b"#"],
                block_comment: None,
                quotes: b"\"'",
                keywords: &[
                    "and", "as", "assert", "async", "await", "break", "class", "continue", "def",
                    "del", "elif", "else", "except", "False", "finally", "for", "from", "global",
                    "if", "import", "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass",
                    "raise", "return", "True", "try", "while", "with", "yield",
                ],
                literals: &["False", "None", "True"],
                key_separators: &[],
                json_keys: false,
                html: false,
            },
            Syntax::Shell => Rules {
                line_comments: &[b"#"],
                block_comment: None,
                quotes: b"\"'",
                keywords: &[
                    "if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done",
                    "case", "esac", "function", "in", "select", "time", "export", "local",
                    "readonly", "declare", "unset", "shift", "return", "exit", "break", "continue",
                ],
                literals: &[],
                key_separators: &[],
                json_keys: false,
                html: false,
            },
            Syntax::Toml => Rules {
                line_comments: &[b"#"],
                block_comment: None,
                quotes: b"\"'",
                keywords: &[],
                literals: &["true", "false"],
                key_separators: b"=",
                json_keys: false,
                html: false,
            },
            Syntax::Yaml => Rules {
                line_comments: &[b"#"],
                block_comment: None,
                quotes: b"\"'",
                keywords: &[],
                literals: &["true", "false", "null", "yes", "no", "on", "off"],
                key_separators: b":",
                json_keys: false,
                html: false,
            },
            Syntax::Css => Rules {
                line_comments: &[],
                block_comment: Some((b"/*", b"*/")),
                quotes: b"\"'",
                keywords: &[],
                literals: &[],
                key_separators: b":",
                json_keys: false,
                html: false,
            },
            Syntax::Html => Rules {
                line_comments: &[],
                block_comment: Some((b"<!--", b"-->")),
                quotes: b"\"'",
                keywords: &[],
                literals: &[],
                key_separators: &[],
                json_keys: false,
                html: true,
            },
            Syntax::Javascript => Rules {
                line_comments: &[b"\x2f\x2f"],
                block_comment: Some((b"/*", b"*/")),
                quotes: b"\"'`",
                keywords: &[
                    "async",
                    "await",
                    "break",
                    "case",
                    "catch",
                    "class",
                    "const",
                    "continue",
                    "debugger",
                    "default",
                    "delete",
                    "do",
                    "else",
                    "export",
                    "extends",
                    "false",
                    "finally",
                    "for",
                    "function",
                    "if",
                    "import",
                    "in",
                    "instanceof",
                    "let",
                    "new",
                    "null",
                    "of",
                    "return",
                    "static",
                    "super",
                    "switch",
                    "this",
                    "throw",
                    "true",
                    "try",
                    "typeof",
                    "undefined",
                    "var",
                    "void",
                    "while",
                    "with",
                    "yield",
                ],
                literals: &["true", "false", "null", "undefined"],
                key_separators: &[],
                json_keys: false,
                html: false,
            },
            Syntax::Plain => Rules {
                line_comments: &[],
                block_comment: None,
                quotes: &[],
                keywords: &[],
                literals: &[],
                key_separators: &[],
                json_keys: false,
                html: false,
            },
        }
    }
}

struct Rules {
    line_comments: &'static [&'static [u8]],
    block_comment: Option<(&'static [u8], &'static [u8])>,
    quotes: &'static [u8],
    keywords: &'static [&'static str],
    literals: &'static [&'static str],
    key_separators: &'static [u8],
    json_keys: bool,
    html: bool,
}

pub(crate) fn highlights(text: &str, syntax: Syntax) -> Vec<(Range<usize>, SyntaxClass)> {
    let rules = syntax.rules();
    let bytes = text.as_bytes();
    let mut highlights = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if line_comment_length(bytes, index, rules.line_comments).is_some() {
            let end = bytes[index..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map(|offset| index + offset)
                .unwrap_or(bytes.len());
            highlights.push((index..end, SyntaxClass::Comment));
            index = end;
            continue;
        }
        if let Some((start, end)) = rules.block_comment
            && bytes[index..].starts_with(start)
        {
            let content_start = index + start.len();
            let comment_end = find_bytes(bytes, content_start, end)
                .map(|offset| offset + end.len())
                .unwrap_or(bytes.len());
            highlights.push((index..comment_end, SyntaxClass::Comment));
            index = comment_end;
            continue;
        }
        if rules.html && bytes[index] == b'<' && !bytes[index..].starts_with(b"<!--") {
            let mut end = index + 1;
            if bytes.get(end) == Some(&b'/') {
                end += 1;
            }
            let name_start = end;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'-' | b'_'))
            {
                end += 1;
            }
            if end > name_start {
                highlights.push((index..end, SyntaxClass::Keyword));
                index = end;
                continue;
            }
        }
        if rules.quotes.contains(&bytes[index]) {
            let quote = bytes[index];
            let mut end = index + 1;
            while end < bytes.len() {
                if bytes[end] == b'\\' {
                    end += 2;
                    continue;
                }
                if bytes[end] == quote {
                    end += 1;
                    break;
                }
                if quote != b'`' && bytes[end] == b'\n' {
                    break;
                }
                end += 1;
            }
            let end = end.min(bytes.len());
            let class = if rules.json_keys {
                let next = skip_space(bytes, end);
                if bytes.get(next) == Some(&b':') {
                    SyntaxClass::Key
                } else {
                    SyntaxClass::String
                }
            } else {
                SyntaxClass::String
            };
            highlights.push((index..end, class));
            index = end;
            continue;
        }
        if bytes[index].is_ascii_digit()
            || (bytes[index] == b'-' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit))
        {
            let mut end = index + 1;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric()
                    || matches!(bytes[end], b'.' | b'_' | b'+' | b'-'))
            {
                end += 1;
            }
            highlights.push((index..end, SyntaxClass::Number));
            index = end;
            continue;
        }
        if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let mut end = index;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            let word = &text[index..end];
            let class = if rules.keywords.contains(&word) {
                Some(SyntaxClass::Keyword)
            } else if rules.literals.contains(&word) {
                Some(SyntaxClass::Literal)
            } else if is_key(bytes, index, rules.key_separators) {
                Some(SyntaxClass::Key)
            } else {
                None
            };
            if let Some(class) = class {
                highlights.push((index..end, class));
            }
            index = end;
            continue;
        }
        index += 1;
    }
    highlights
}

fn line_comment_length(
    bytes: &[u8],
    index: usize,
    comments: &'static [&'static [u8]],
) -> Option<usize> {
    comments
        .iter()
        .find(|comment| bytes[index..].starts_with(comment))
        .map(|comment| comment.len())
}

fn find_bytes(bytes: &[u8], start: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(start);
    }
    (start..bytes.len().saturating_sub(needle.len() - 1))
        .find(|index| bytes[*index..].starts_with(needle))
}

fn skip_space(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    index
}

fn is_key(bytes: &[u8], index: usize, separators: &'static [u8]) -> bool {
    if separators.is_empty() {
        return false;
    }
    let mut start = index;
    let mut at_line_start = true;
    while start > 0 {
        start -= 1;
        match bytes[start] {
            b'\n' => break,
            b' ' | b'\t' | b'\r' => {}
            _ => {
                at_line_start = false;
                break;
            }
        }
    }
    if !at_line_start {
        return false;
    }
    let mut end = index;
    while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
        end += 1;
    }
    let next = skip_space(bytes, end);
    next < bytes.len() && separators.contains(&bytes[next])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_extensions() {
        assert_eq!(Syntax::detect(Path::new("a.json")), Syntax::Json);
        assert_eq!(Syntax::detect(Path::new("a.rs")), Syntax::Rust);
        assert_eq!(Syntax::detect(Path::new("a.ts")), Syntax::Javascript);
        assert_eq!(Syntax::detect(Path::new("a")), Syntax::Plain);
    }

    #[test]
    fn highlights_json_tokens() {
        let text = r#"{"name": "value", "count": 42, "ok": true}"#;
        let found = highlights(text, Syntax::Json);
        assert!(found.iter().any(|(_, class)| *class == SyntaxClass::Key));
        assert!(found.iter().any(|(_, class)| *class == SyntaxClass::String));
        assert!(found.iter().any(|(_, class)| *class == SyntaxClass::Number));
        assert!(
            found
                .iter()
                .any(|(_, class)| *class == SyntaxClass::Literal)
        );
        for (range, _) in &found {
            assert!(text.is_char_boundary(range.start));
            assert!(text.is_char_boundary(range.end));
        }
    }

    #[test]
    fn highlights_rust_comments_strings_and_keywords() {
        let text = "\x2f\x2f comment\nlet name = \"value\";\n";
        let found = highlights(text, Syntax::Rust);
        assert!(
            found
                .iter()
                .any(|(_, class)| *class == SyntaxClass::Comment)
        );
        assert!(found.iter().any(|(_, class)| *class == SyntaxClass::String));
        assert!(
            found
                .iter()
                .any(|(_, class)| *class == SyntaxClass::Keyword)
        );
    }
}
