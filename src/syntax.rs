use std::ops::Range;
use std::path::Path;

mod languages;

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
    C,
    Go,
    Java,
    CSharp,
    Swift,
    Kotlin,
    Ruby,
    Php,
    Lua,
    Sql,
    Haskell,
    Nix,
    R,
    Perl,
    Dockerfile,
    Makefile,
    Cmake,
    Ini,
    Diff,
    Plain,
}

impl Syntax {
    pub(crate) fn detect(path: &Path) -> Self {
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            match name.to_ascii_lowercase().as_str() {
                "dockerfile" | "containerfile" => return Syntax::Dockerfile,
                "makefile" | "gnumakefile" | "justfile" => return Syntax::Makefile,
                "cmakelists.txt" => return Syntax::Cmake,
                "cargo.lock" => return Syntax::Toml,
                _ => {}
            }
        }
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            return Syntax::Plain;
        };
        match extension.to_ascii_lowercase().as_str() {
            "json" => Syntax::Json,
            "rs" => Syntax::Rust,
            "py" | "pyw" => Syntax::Python,
            "sh" | "bash" | "zsh" | "fish" | "ksh" => Syntax::Shell,
            "toml" | "lock" => Syntax::Toml,
            "yaml" | "yml" => Syntax::Yaml,
            "css" | "scss" | "sass" | "less" => Syntax::Css,
            "html" | "htm" | "xml" | "xhtml" | "svg" | "vue" | "svelte" => Syntax::Html,
            "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" | "mts" | "cts" => Syntax::Javascript,
            "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hxx" | "hh" | "m" | "mm" | "proto" => {
                Syntax::C
            }
            "go" => Syntax::Go,
            "java" | "gradle" => Syntax::Java,
            "cs" => Syntax::CSharp,
            "kt" | "kts" => Syntax::Kotlin,
            "swift" => Syntax::Swift,
            "rb" | "rake" => Syntax::Ruby,
            "php" | "phtml" => Syntax::Php,
            "lua" => Syntax::Lua,
            "sql" => Syntax::Sql,
            "hs" | "lhs" => Syntax::Haskell,
            "nix" => Syntax::Nix,
            "r" => Syntax::R,
            "pl" | "pm" => Syntax::Perl,
            "mk" | "mak" => Syntax::Makefile,
            "cmake" => Syntax::Cmake,
            "ini" | "cfg" | "conf" | "service" | "desktop" => Syntax::Ini,
            "diff" | "patch" => Syntax::Diff,
            _ => Syntax::Plain,
        }
    }

    pub(crate) fn detect_with_text(path: &Path, text: &str) -> Self {
        let syntax = Self::detect(path);
        if syntax != Syntax::Plain {
            return syntax;
        }
        let Some(shebang) = text
            .lines()
            .next()
            .and_then(|line| line.trim().strip_prefix("#!"))
        else {
            return Syntax::Plain;
        };
        let shebang = shebang.to_ascii_lowercase();
        if shebang.contains("python") {
            Syntax::Python
        } else if shebang.contains("node") || shebang.contains("deno") || shebang.contains("bun") {
            Syntax::Javascript
        } else if shebang.contains("ruby") {
            Syntax::Ruby
        } else if shebang.contains("perl") {
            Syntax::Perl
        } else if shebang.contains("php") {
            Syntax::Php
        } else if shebang.contains("lua") {
            Syntax::Lua
        } else if shebang.contains("rscript") {
            Syntax::R
        } else if shebang.contains("bash")
            || shebang.contains("zsh")
            || shebang.contains("fish")
            || shebang.contains("sh")
        {
            Syntax::Shell
        } else {
            Syntax::Plain
        }
    }

    pub(crate) fn from_label(label: &str) -> Self {
        match label.trim().to_ascii_lowercase().as_str() {
            "json" => Syntax::Json,
            "rs" | "rust" => Syntax::Rust,
            "py" | "python" => Syntax::Python,
            "sh" | "bash" | "zsh" | "fish" | "ksh" | "shell" => Syntax::Shell,
            "toml" => Syntax::Toml,
            "yml" | "yaml" => Syntax::Yaml,
            "css" | "scss" | "less" => Syntax::Css,
            "html" | "htm" | "xml" | "svg" => Syntax::Html,
            "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" | "mts" | "cts" | "javascript"
            | "typescript" => Syntax::Javascript,
            "c" | "h" | "cpp" | "c++" | "cxx" | "cc" | "hpp" | "objective-c" => Syntax::C,
            "go" | "golang" => Syntax::Go,
            "java" => Syntax::Java,
            "cs" | "csharp" | "c#" => Syntax::CSharp,
            "kt" | "kts" | "kotlin" => Syntax::Kotlin,
            "swift" => Syntax::Swift,
            "rb" | "ruby" | "rake" => Syntax::Ruby,
            "php" => Syntax::Php,
            "lua" => Syntax::Lua,
            "sql" => Syntax::Sql,
            "hs" | "haskell" => Syntax::Haskell,
            "nix" => Syntax::Nix,
            "r" => Syntax::R,
            "pl" | "perl" => Syntax::Perl,
            "dockerfile" | "docker" => Syntax::Dockerfile,
            "makefile" | "make" | "mk" => Syntax::Makefile,
            "cmake" => Syntax::Cmake,
            "ini" | "cfg" | "conf" => Syntax::Ini,
            "diff" | "patch" => Syntax::Diff,
            _ => Syntax::Plain,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Syntax::Json => "JSON",
            Syntax::Rust => "Rust",
            Syntax::Python => "Python",
            Syntax::Shell => "Shell",
            Syntax::Toml => "TOML",
            Syntax::Yaml => "YAML",
            Syntax::Css => "CSS",
            Syntax::Html => "HTML/XML",
            Syntax::Javascript => "JavaScript/TypeScript",
            Syntax::C => "C/C++",
            Syntax::Go => "Go",
            Syntax::Java => "Java",
            Syntax::CSharp => "C#",
            Syntax::Swift => "Swift",
            Syntax::Kotlin => "Kotlin",
            Syntax::Ruby => "Ruby",
            Syntax::Php => "PHP",
            Syntax::Lua => "Lua",
            Syntax::Sql => "SQL",
            Syntax::Haskell => "Haskell",
            Syntax::Nix => "Nix",
            Syntax::R => "R",
            Syntax::Perl => "Perl",
            Syntax::Dockerfile => "Dockerfile",
            Syntax::Makefile => "Makefile",
            Syntax::Cmake => "CMake",
            Syntax::Ini => "INI",
            Syntax::Diff => "Diff",
            Syntax::Plain => "Text",
        }
    }

    fn rules(self) -> languages::Rules {
        languages::rules(self)
    }
}

pub(crate) fn highlights(text: &str, syntax: Syntax) -> Vec<(Range<usize>, SyntaxClass)> {
    let rules = syntax.rules();
    let bytes = text.as_bytes();
    let mut highlights = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if rules.diff && (index == 0 || bytes[index - 1] == b'\n') {
            let class = match bytes[index] {
                b'+' => Some(SyntaxClass::String),
                b'-' => Some(SyntaxClass::Comment),
                b'@' => Some(SyntaxClass::Keyword),
                _ => None,
            };
            if let Some(class) = class {
                let end = bytes[index..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map(|offset| index + offset)
                    .unwrap_or(bytes.len());
                highlights.push((index..end, class));
                index = end;
                continue;
            }
        }
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
    fn detects_common_extensions_and_names() {
        assert_eq!(Syntax::detect(Path::new("a.json")), Syntax::Json);
        assert_eq!(Syntax::detect(Path::new("a.rs")), Syntax::Rust);
        assert_eq!(Syntax::detect(Path::new("a.ts")), Syntax::Javascript);
        assert_eq!(Syntax::detect(Path::new("a.go")), Syntax::Go);
        assert_eq!(Syntax::detect(Path::new("a.cpp")), Syntax::C);
        assert_eq!(Syntax::detect(Path::new("a.sql")), Syntax::Sql);
        assert_eq!(Syntax::detect(Path::new("Dockerfile")), Syntax::Dockerfile);
        assert_eq!(Syntax::detect(Path::new("Makefile")), Syntax::Makefile);
        assert_eq!(Syntax::detect(Path::new("CMakeLists.txt")), Syntax::Cmake);
        assert_eq!(Syntax::detect(Path::new("a")), Syntax::Plain);
    }

    #[test]
    fn detects_shebangs_and_fence_labels() {
        assert_eq!(
            Syntax::detect_with_text(Path::new("script"), "#!/usr/bin/env python3\nprint(1)"),
            Syntax::Python
        );
        assert_eq!(
            Syntax::detect_with_text(Path::new("script"), "#!/bin/bash\necho hi"),
            Syntax::Shell
        );
        assert_eq!(Syntax::from_label("rust"), Syntax::Rust);
        assert_eq!(Syntax::from_label("cpp"), Syntax::C);
        assert_eq!(Syntax::from_label("ts"), Syntax::Javascript);
        assert_eq!(Syntax::from_label("unknown"), Syntax::Plain);
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

    #[test]
    fn highlights_diff_lines() {
        let text = "--- a\n+++ b\n@@ -1 +1 @@\n-old\n+new\n";
        let found = highlights(text, Syntax::Diff);
        assert!(
            found
                .iter()
                .any(|(range, class)| *class == SyntaxClass::String
                    && &text[range.clone()] == "+new")
        );
        assert!(
            found
                .iter()
                .any(|(range, class)| *class == SyntaxClass::Comment
                    && &text[range.clone()] == "-old")
        );
    }
}
