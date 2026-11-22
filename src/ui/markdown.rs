use gpui::prelude::*;
use gpui::{
    AnyElement, FontStyle, FontWeight, HighlightStyle, StrikethroughStyle, StyledText,
    UnderlineStyle, div, px, rgb,
};

use crate::markdown::{Block, Inline, InlineStyle};
use crate::syntax::Syntax;
use crate::theme::Theme;
use crate::ui::code::code_view;

pub(crate) fn markdown_view(blocks: &[Block], theme: Theme) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2()
        .children(blocks.iter().map(|block| block_view(block, theme)))
        .into_any_element()
}

fn block_view(block: &Block, theme: Theme) -> AnyElement {
    match block {
        Block::Heading { level, content } => {
            let size = match level {
                1 => 20.0,
                2 => 18.0,
                3 => 16.0,
                4 => 14.0,
                5 => 13.0,
                _ => 12.0,
            };
            div()
                .w_full()
                .text_size(px(size))
                .line_height(px(size + 6.0))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(theme.text))
                .child(inline_view(content, theme))
                .into_any_element()
        }
        Block::Paragraph(content) => div()
            .w_full()
            .text_sm()
            .line_height(px(20.))
            .text_color(rgb(theme.text))
            .child(inline_view(content, theme))
            .into_any_element(),
        Block::Code { language, code } => {
            let syntax = language
                .as_deref()
                .map(Syntax::from_label)
                .unwrap_or(Syntax::Plain);
            let mut body = div()
                .w_full()
                .flex()
                .flex_col()
                .gap_1()
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(theme.border))
                .bg(rgb(theme.input_bg))
                .p_2();
            if syntax != Syntax::Plain {
                body = body.child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.dim))
                        .child(syntax.label()),
                );
            }
            body.child(code_view(code, syntax, theme, false))
                .into_any_element()
        }
        Block::Quote(inner) => div()
            .w_full()
            .border_l_2()
            .border_color(rgb(theme.accent))
            .pl_3()
            .flex()
            .flex_col()
            .gap_1()
            .children(inner.iter().map(|block| block_view(block, theme)))
            .into_any_element(),
        Block::List { start, items } => div()
            .w_full()
            .flex()
            .flex_col()
            .gap_1()
            .children(items.iter().enumerate().map(|(index, item)| {
                let marker = match start {
                    Some(start) => format!("{}.", start + index as u64),
                    None => "•".to_string(),
                };
                div()
                    .flex()
                    .items_start()
                    .gap_2()
                    .w_full()
                    .child(
                        div()
                            .w(px(18.))
                            .flex_none()
                            .text_right()
                            .text_sm()
                            .line_height(px(20.))
                            .text_color(rgb(theme.muted))
                            .child(marker),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .flex()
                            .flex_col()
                            .gap_1()
                            .children(item.iter().map(|block| block_view(block, theme))),
                    )
            }))
            .into_any_element(),
        Block::Rule => div()
            .w_full()
            .h(px(1.))
            .bg(rgb(theme.border))
            .into_any_element(),
        Block::Table { header, rows } => {
            let mut table = div()
                .w_full()
                .flex()
                .flex_col()
                .rounded(px(4.))
                .border_1()
                .border_color(rgb(theme.border))
                .overflow_hidden();
            if !header.is_empty() {
                table = table.child(table_row(header, true, theme));
            }
            for row in rows {
                table = table.child(table_row(row, false, theme));
            }
            table.into_any_element()
        }
    }
}

fn table_row(cells: &[Vec<Inline>], header: bool, theme: Theme) -> AnyElement {
    let mut row = div()
        .w_full()
        .flex()
        .border_b_1()
        .border_color(rgb(theme.border));
    if header {
        row = row.bg(rgb(theme.panel2));
    }
    row.children(cells.iter().map(|cell| {
        div()
            .flex_1()
            .min_w(px(0.))
            .p_2()
            .text_xs()
            .line_height(px(18.))
            .font_weight(if header {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(rgb(if header { theme.text } else { theme.muted }))
            .child(inline_view(cell, theme))
    }))
    .into_any_element()
}

fn inline_view(inlines: &[Inline], theme: Theme) -> StyledText {
    let mut text = String::new();
    let mut highlights = Vec::new();
    for inline in inlines {
        let start = text.len();
        text.push_str(&inline.text);
        let end = text.len();
        if start == end {
            continue;
        }
        let style = highlight_style(inline.style, theme);
        if style != HighlightStyle::default() {
            highlights.push((start..end, style));
        }
    }
    if highlights.is_empty() {
        StyledText::new(text)
    } else {
        StyledText::new(text).with_highlights(highlights)
    }
}

fn highlight_style(style: InlineStyle, theme: Theme) -> HighlightStyle {
    let mut highlight = HighlightStyle::default();
    if style.bold {
        highlight.font_weight = Some(FontWeight::SEMIBOLD);
    }
    if style.italic {
        highlight.font_style = Some(FontStyle::Italic);
    }
    if style.code {
        highlight.color = Some(rgb(theme.amber).into());
        highlight.background_color = Some(rgb(theme.panel2).into());
    }
    if style.link {
        highlight.color = Some(rgb(theme.accent).into());
        highlight.underline = Some(UnderlineStyle {
            thickness: px(1.),
            color: Some(rgb(theme.accent).into()),
            ..Default::default()
        });
    }
    if style.strikethrough {
        highlight.strikethrough = Some(StrikethroughStyle {
            thickness: px(1.),
            color: Some(rgb(theme.muted).into()),
        });
    }
    highlight
}
