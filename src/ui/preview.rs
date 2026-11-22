use std::path::Path;

use gpui::prelude::*;
use gpui::{AnyElement, Context, ObjectFit, StyledText, div, img, px, rgb};

use crate::app::AppView;
use crate::preview::PreviewContent;
use crate::state::PreviewView;
use crate::syntax::Syntax;
use crate::theme::Theme;
use crate::ui::code::code_view;
use crate::ui::markdown::markdown_view;

impl AppView {
    pub(crate) fn render_preview_pane(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        let Some(preview) = &self.preview else {
            return div().into_any_element();
        };
        let title = preview.title.clone();
        let has_path = preview.path.is_some();
        let language = preview_language(preview);
        let body: AnyElement = match &preview.content {
            PreviewContent::Loading => div()
                .text_sm()
                .text_color(rgb(theme.muted))
                .child("Preparing preview…")
                .into_any_element(),
            PreviewContent::Text(text) => {
                let path = preview.path.as_deref().unwrap_or_else(|| Path::new(""));
                let syntax = Syntax::detect_with_text(path, text);
                if syntax == Syntax::Plain {
                    plain_text(text, theme)
                } else {
                    code_view(text, syntax, theme, true)
                }
            }
            PreviewContent::Markdown(blocks) => markdown_view(blocks, theme),
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
        let mut header = div()
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
            );
        if let Some(language) = language {
            header = header.child(
                div()
                    .flex_none()
                    .text_xs()
                    .text_color(rgb(theme.dim))
                    .child(language),
            );
        }
        header = header
            .child(
                self.btn("preview-open", "Open", has_path, cx, |this, _, cx| {
                    this.open_preview_file(cx)
                }),
            )
            .child(self.btn("preview-close", "×", true, cx, |this, _, cx| {
                this.close_preview(cx)
            }));
        div()
            .h(gpui::relative(0.32))
            .flex_none()
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(rgb(theme.border))
            .bg(rgb(theme.panel))
            .child(header)
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

fn plain_text(text: &str, theme: Theme) -> AnyElement {
    div()
        .w_full()
        .font_family("monospace")
        .text_xs()
        .line_height(px(18.))
        .text_color(rgb(theme.text))
        .child(StyledText::new(text.to_string()))
        .into_any_element()
}

fn preview_language(preview: &PreviewView) -> Option<&'static str> {
    match &preview.content {
        PreviewContent::Markdown(_) => Some("Markdown"),
        PreviewContent::Text(text) => {
            let path = preview.path.as_deref().unwrap_or_else(|| Path::new(""));
            let syntax = Syntax::detect_with_text(path, text);
            (syntax != Syntax::Plain).then_some(syntax.label())
        }
        _ => None,
    }
}
