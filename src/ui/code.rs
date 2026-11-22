use gpui::prelude::*;
use gpui::{AnyElement, HighlightStyle, StyledText, div, px, rgb};

use crate::syntax::{self, Syntax, SyntaxClass};
use crate::theme::Theme;

pub(crate) fn code_view(
    code: &str,
    syntax: Syntax,
    theme: Theme,
    numbered: bool,
    wrap: bool,
) -> AnyElement {
    let code = expand_tabs(code);
    let styled = styled_text(&code, syntax, theme);
    let line_height = px(16.);
    let body = div()
        .font_family("monospace")
        .text_xs()
        .line_height(line_height)
        .child(styled);
    if wrap {
        div()
            .w_full()
            .font_family("monospace")
            .text_xs()
            .line_height(line_height)
            .text_color(rgb(theme.text))
            .child(styled_text(&code, syntax, theme))
            .into_any_element()
    } else if numbered {
        let line_count = code.split('\n').count().max(1);
        let width = line_count.to_string().len();
        let numbers = (1..=line_count)
            .map(|line| format!("{line:>width$}"))
            .collect::<Vec<_>>()
            .join("\n");
        div()
            .flex()
            .items_start()
            .gap_2()
            .w_full()
            .child(
                div()
                    .flex_none()
                    .pr_2()
                    .border_r_1()
                    .border_color(rgb(theme.border))
                    .text_right()
                    .font_family("monospace")
                    .text_xs()
                    .line_height(line_height)
                    .text_color(rgb(theme.dim))
                    .whitespace_nowrap()
                    .child(numbers),
            )
            .child(
                div()
                    .id("preview-code-hscroll")
                    .flex_1()
                    .min_w(px(0.))
                    .overflow_x_scroll()
                    .child(body.whitespace_nowrap()),
            )
            .into_any_element()
    } else {
        div().w_full().child(body).into_any_element()
    }
}

fn styled_text(text: &str, syntax: Syntax, theme: Theme) -> StyledText {
    let highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> =
        syntax::highlights(text, syntax)
            .into_iter()
            .map(|(range, class)| {
                (
                    range,
                    HighlightStyle::color(rgb(color_for(theme, class)).into()),
                )
            })
            .collect();
    if highlights.is_empty() {
        StyledText::new(text.to_string())
    } else {
        StyledText::new(text.to_string()).with_highlights(highlights)
    }
}

fn color_for(theme: Theme, class: SyntaxClass) -> u32 {
    match class {
        SyntaxClass::Keyword | SyntaxClass::Key => theme.accent,
        SyntaxClass::String => theme.amber,
        SyntaxClass::Number | SyntaxClass::Literal => theme.purple,
        SyntaxClass::Comment => theme.dim,
    }
}

fn expand_tabs(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut column = 0;
    for character in text.chars() {
        match character {
            '\t' => {
                let spaces = 4 - column % 4;
                for _ in 0..spaces {
                    output.push(' ');
                }
                column += spaces;
            }
            '\n' => {
                output.push('\n');
                column = 0;
            }
            _ => {
                output.push(character);
                column += 1;
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_tabs_at_four_column_stops() {
        assert_eq!(expand_tabs("a\tb"), "a   b");
        assert_eq!(expand_tabs("abcd\te"), "abcd    e");
        assert_eq!(expand_tabs("\tline\n\tline"), "    line\n    line");
    }
}
