use std::iter::Peekable;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Block {
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    Code {
        language: Option<String>,
        code: String,
    },
    Quote(Vec<Block>),
    List {
        start: Option<u64>,
        items: Vec<Vec<Block>>,
    },
    Rule,
    Table {
        header: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Inline {
    pub(crate) text: String,
    pub(crate) style: InlineStyle,
    pub(crate) link: Option<String>,
    pub(crate) image: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct InlineStyle {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) code: bool,
    pub(crate) link: bool,
    pub(crate) strikethrough: bool,
}

pub(crate) fn parse(source: &str) -> Vec<Block> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_MATH;
    let mut events = Parser::new_ext(source, options).peekable();
    parse_blocks(&mut events, None)
}

fn parse_blocks(events: &mut Peekable<Parser<'_>>, end: Option<TagEnd>) -> Vec<Block> {
    let mut blocks = Vec::new();
    while let Some(event) = events.next() {
        match event {
            Event::End(tag) if Some(tag) == end => break,
            Event::End(_) => {}
            Event::Start(Tag::Paragraph) => {
                blocks.push(Block::Paragraph(parse_inlines_bounded(
                    events,
                    Some(TagEnd::Paragraph),
                    false,
                )));
            }
            Event::Start(Tag::Heading { level, .. }) => {
                let content = parse_inlines_bounded(events, Some(TagEnd::Heading(level)), false);
                blocks.push(Block::Heading {
                    level: level as u8,
                    content,
                });
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    CodeBlockKind::Fenced(info) => info
                        .split_whitespace()
                        .next()
                        .filter(|value| !value.is_empty())
                        .map(str::to_string),
                    CodeBlockKind::Indented => None,
                };
                blocks.push(Block::Code {
                    language,
                    code: parse_code(events),
                });
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                let quote = parse_blocks(events, Some(TagEnd::BlockQuote(kind)));
                blocks.push(Block::Quote(quote));
            }
            Event::Start(Tag::List(start)) => {
                blocks.push(Block::List {
                    start,
                    items: parse_list(events),
                });
            }
            Event::Start(Tag::Table(_)) => blocks.push(parse_table(events)),
            Event::Start(Tag::HtmlBlock) => {
                let mut raw = String::new();
                for event in events.by_ref() {
                    match event {
                        Event::End(TagEnd::HtmlBlock) => break,
                        Event::Html(text) | Event::Text(text) => raw.push_str(&text),
                        _ => {}
                    }
                }
                if !raw.trim().is_empty() {
                    blocks.push(Block::Paragraph(vec![Inline {
                        text: raw,
                        style: InlineStyle::default(),
                        link: None,
                        image: None,
                    }]));
                }
            }
            Event::Rule => blocks.push(Block::Rule),
            _ => {}
        }
    }
    blocks
}

fn parse_code(events: &mut Peekable<Parser<'_>>) -> String {
    let mut code = String::new();
    for event in events.by_ref() {
        match event {
            Event::End(TagEnd::CodeBlock) => break,
            Event::Text(text) | Event::Code(text) => code.push_str(&text),
            Event::SoftBreak | Event::HardBreak => code.push('\n'),
            _ => {}
        }
    }
    code
}

fn parse_list(events: &mut Peekable<Parser<'_>>) -> Vec<Vec<Block>> {
    let mut items = Vec::new();
    while let Some(event) = events.next() {
        match event {
            Event::Start(Tag::Item) => items.push(parse_item(events)),
            Event::End(TagEnd::List(_)) => break,
            _ => {}
        }
    }
    items
}

fn parse_table(events: &mut Peekable<Parser<'_>>) -> Block {
    let mut header = Vec::new();
    let mut rows = Vec::new();
    while let Some(event) = events.next() {
        match event {
            Event::Start(Tag::TableHead) => {
                header = parse_table_cells(events, TagEnd::TableHead);
            }
            Event::Start(Tag::TableRow) => {
                rows.push(parse_table_cells(events, TagEnd::TableRow));
            }
            Event::End(TagEnd::Table) => break,
            _ => {}
        }
    }
    Block::Table { header, rows }
}

fn parse_table_cells(events: &mut Peekable<Parser<'_>>, end: TagEnd) -> Vec<Vec<Inline>> {
    let mut cells = Vec::new();
    while let Some(event) = events.next() {
        match event {
            Event::Start(Tag::TableCell) => {
                cells.push(parse_inlines_bounded(
                    events,
                    Some(TagEnd::TableCell),
                    false,
                ));
            }
            Event::End(tag) if tag == end => break,
            _ => {}
        }
    }
    cells
}

fn parse_item(events: &mut Peekable<Parser<'_>>) -> Vec<Block> {
    let mut blocks = Vec::new();
    let content = parse_inlines_bounded(events, Some(TagEnd::Item), true);
    if !content.is_empty() {
        blocks.push(Block::Paragraph(content));
    }
    blocks.extend(parse_blocks(events, Some(TagEnd::Item)));
    blocks
}

fn parse_inlines_bounded(
    events: &mut Peekable<Parser<'_>>,
    end: Option<TagEnd>,
    stop_at_block: bool,
) -> Vec<Inline> {
    let mut inlines = Vec::new();
    let mut style = InlineStyle::default();
    let mut link: Option<String> = None;
    let mut image: Option<String> = None;
    let mut stack: Vec<(InlineStyle, Option<String>, Option<String>)> = Vec::new();
    loop {
        if stop_at_block {
            match events.peek() {
                Some(Event::Start(tag)) if is_block_start(tag) => break,
                Some(Event::Rule) => break,
                Some(Event::End(tag)) if Some(*tag) == end => break,
                _ => {}
            }
        }
        let Some(event) = events.next() else {
            break;
        };
        match event {
            Event::End(tag) if Some(tag) == end => break,
            Event::Start(Tag::Emphasis) => {
                stack.push((style, link.clone(), image.clone()));
                style.italic = true;
            }
            Event::Start(Tag::Strong) => {
                stack.push((style, link.clone(), image.clone()));
                style.bold = true;
            }
            Event::Start(Tag::Strikethrough) => {
                stack.push((style, link.clone(), image.clone()));
                style.strikethrough = true;
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                stack.push((style, link.clone(), image.clone()));
                style.link = true;
                link = Some(dest_url.to_string());
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                stack.push((style, link.clone(), image.clone()));
                style.link = true;
                link = Some(dest_url.to_string());
                image = Some(dest_url.to_string());
            }
            Event::End(TagEnd::Emphasis)
            | Event::End(TagEnd::Strong)
            | Event::End(TagEnd::Strikethrough)
            | Event::End(TagEnd::Link)
            | Event::End(TagEnd::Image) => {
                if let Some((previous, previous_link, previous_image)) = stack.pop() {
                    style = previous;
                    link = previous_link;
                    image = previous_image;
                }
            }
            Event::Text(text) => {
                push_inline(
                    &mut inlines,
                    &text,
                    style,
                    link.as_deref(),
                    image.as_deref(),
                );
            }
            Event::Code(text) => {
                let mut code_style = style;
                code_style.code = true;
                push_inline(
                    &mut inlines,
                    &text,
                    code_style,
                    link.as_deref(),
                    image.as_deref(),
                );
            }
            Event::InlineMath(text) | Event::DisplayMath(text) => {
                let mut code_style = style;
                code_style.code = true;
                push_inline(
                    &mut inlines,
                    &text,
                    code_style,
                    link.as_deref(),
                    image.as_deref(),
                );
            }
            Event::SoftBreak => {
                push_inline(&mut inlines, " ", style, link.as_deref(), image.as_deref());
            }
            Event::HardBreak => {
                push_inline(&mut inlines, "\n", style, link.as_deref(), image.as_deref());
            }
            Event::TaskListMarker(checked) => {
                push_inline(
                    &mut inlines,
                    if checked { "☑ " } else { "☐ " },
                    style,
                    link.as_deref(),
                    image.as_deref(),
                );
            }
            Event::FootnoteReference(label) => {
                push_inline(
                    &mut inlines,
                    &format!("[^{label}]"),
                    style,
                    link.as_deref(),
                    image.as_deref(),
                );
            }
            Event::InlineHtml(text) => {
                push_inline(
                    &mut inlines,
                    &text,
                    style,
                    link.as_deref(),
                    image.as_deref(),
                );
            }
            _ => {}
        }
    }
    inlines
}

fn is_block_start(tag: &Tag<'_>) -> bool {
    matches!(
        tag,
        Tag::Paragraph
            | Tag::Heading { .. }
            | Tag::CodeBlock(_)
            | Tag::BlockQuote(_)
            | Tag::List(_)
            | Tag::Item
            | Tag::Table(_)
            | Tag::HtmlBlock
            | Tag::FootnoteDefinition(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition
    )
}

fn push_inline(
    inlines: &mut Vec<Inline>,
    text: &str,
    style: InlineStyle,
    link: Option<&str>,
    image: Option<&str>,
) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = inlines.last_mut()
        && last.style == style
        && last.link.as_deref() == link
        && last.image.as_deref() == image
    {
        last.text.push_str(text);
        return;
    }
    inlines.push(Inline {
        text: text.to_string(),
        style,
        link: link.map(str::to_string),
        image: image.map(str::to_string),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(inlines: &[Inline]) -> String {
        inlines.iter().map(|inline| inline.text.as_str()).collect()
    }

    #[test]
    fn parses_headings_paragraphs_and_inline_styles() {
        let blocks = parse(
            "# Title\n\nHello **bold** *italic* `code` [link](https://example.com) ~~gone~~.\n",
        );
        match &blocks[0] {
            Block::Heading { level, content } => {
                assert_eq!(*level, 1);
                assert_eq!(plain(content), "Title");
            }
            other => panic!("expected heading, got {other:?}"),
        }
        let Block::Paragraph(content) = &blocks[1] else {
            panic!("expected paragraph");
        };
        assert!(content.iter().any(|inline| inline.style.bold));
        assert!(content.iter().any(|inline| inline.style.italic));
        assert!(content.iter().any(|inline| inline.style.code));
        assert!(content.iter().any(|inline| inline.style.link));
        assert!(content.iter().any(|inline| inline.style.strikethrough));
    }

    #[test]
    fn parses_code_lists_quotes_and_tables() {
        let source = "```rust\nfn main() {}\n```\n\n- one\n- two\n\n> quoted\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";
        let blocks = parse(source);
        assert!(matches!(
            &blocks[0],
            Block::Code { language: Some(language), code } if language == "rust" && code.contains("fn main")
        ));
        let Block::List { items, .. } = &blocks[1] else {
            panic!("expected list");
        };
        assert_eq!(items.len(), 2);
        assert!(matches!(&blocks[2], Block::Quote(_)));
        let Block::Table { header, rows } = &blocks[3] else {
            panic!("expected table: {blocks:?}");
        };
        assert_eq!(header.len(), 2);
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn parses_task_lists() {
        let blocks = parse("- [x] done\n- [ ] todo\n");
        let Block::List { items, .. } = &blocks[0] else {
            panic!("expected list");
        };
        let Block::Paragraph(content) = &items[0][0] else {
            panic!("expected paragraph");
        };
        assert!(plain(content).starts_with("☑ "));
    }
}
