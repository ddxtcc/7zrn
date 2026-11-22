use std::path::Path;

use gpui::prelude::*;
use gpui::{AnyElement, Context, HighlightStyle, ObjectFit, StyledText, div, img, px, rgb};

use crate::app::AppView;
use crate::preview::PreviewContent;
use crate::syntax::{self, Syntax, SyntaxClass};
use crate::theme::Theme;

impl AppView {
    pub(crate) fn render_preview_pane(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        let Some(preview) = &self.preview else {
            return div().into_any_element();
        };
        let title = preview.title.clone();
        let has_path = preview.path.is_some();
        let body: AnyElement = match &preview.content {
            PreviewContent::Loading => div()
                .text_sm()
                .text_color(rgb(theme.muted))
                .child("Preparing preview…")
                .into_any_element(),
            PreviewContent::Text(text) => {
                let path = preview.path.as_deref().unwrap_or_else(|| Path::new(""));
                highlighted_text(text, path, theme)
            }
            PreviewContent::Image => {
                if let Some(path) = &preview.path {
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            img(path.clone())
                                .w_full()
                                .h_full()
                                .object_fit(ObjectFit::Contain),
                        )
                        .into_any_element()
                } else {
                    div()
                        .text_sm()
                        .text_color(rgb(theme.muted))
                        .child("Image not available")
                        .into_any_element()
                }
            }
            PreviewContent::Unsupported(message) => div()
                .text_sm()
                .text_color(rgb(theme.muted))
                .child(message.clone())
                .into_any_element(),
            PreviewContent::Error(error) => div()
                .text_sm()
                .text_color(rgb(theme.red))
                .child(error.clone())
                .into_any_element(),
        };
        div()
            .h(gpui::relative(0.32))
            .flex_none()
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(rgb(theme.border))
            .bg(rgb(theme.panel))
            .child(
                div()
                    .h(px(26.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .border_b_1()
                    .border_color(rgb(theme.border))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(theme.accent))
                            .child("PREVIEW"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .truncate()
                            .text_xs()
                            .text_color(rgb(theme.muted))
                            .child(title),
                    )
                    .child(
                        self.btn("preview-open", "Open", has_path, cx, |this, _, cx| {
                            this.open_preview_file(cx)
                        }),
                    )
                    .child(self.btn("preview-close", "×", true, cx, |this, _, cx| {
                        this.close_preview(cx)
                    })),
            )
            .child(
                div()
                    .id("preview-body")
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_y_scroll()
                    .p_2()
                    .child(body),
            )
            .into_any_element()
    }
}

fn highlighted_text(text: &str, path: &Path, theme: Theme) -> AnyElement {
    let syntax = Syntax::detect(path);
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
    let styled = if highlights.is_empty() {
        StyledText::new(text.to_string())
    } else {
        StyledText::new(text.to_string()).with_highlights(highlights)
    };
    div()
        .w_full()
        .font_family("monospace")
        .text_xs()
        .text_color(rgb(theme.text))
        .child(styled)
        .into_any_element()
}

fn color_for(theme: Theme, class: SyntaxClass) -> u32 {
    match class {
        SyntaxClass::Keyword | SyntaxClass::Key => theme.accent,
        SyntaxClass::String => theme.amber,
        SyntaxClass::Number | SyntaxClass::Literal => theme.purple,
        SyntaxClass::Comment => theme.dim,
    }
}
