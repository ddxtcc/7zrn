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
                    code_view(text, syntax, theme, !self.code_wrap, self.code_wrap)
                }
            }
            PreviewContent::Markdown { blocks, .. } => {
                markdown_view(blocks, theme, preview.path.as_deref(), cx)
            }
            PreviewContent::Image { path } => image_view(path, self.image_zoom),
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
        if matches!(preview.content, PreviewContent::Image { .. }) {
            header = header
                .child(self.btn("image-zoom-out", "−", true, cx, |this, _, cx| {
                    this.set_image_zoom(-0.25, cx)
                }))
                .child(
                    div()
                        .flex_none()
                        .text_xs()
                        .text_color(rgb(theme.muted))
                        .child(format!("{}%", (self.image_zoom * 100.0).round() as i32)),
                )
                .child(self.btn("image-zoom-in", "+", true, cx, |this, _, cx| {
                    this.set_image_zoom(0.25, cx)
                }));
        }
        let is_code = match &preview.content {
            PreviewContent::Text(text) => {
                let path = preview.path.as_deref().unwrap_or_else(|| Path::new(""));
                Syntax::detect_with_text(path, text) != Syntax::Plain
            }
            _ => false,
        };
        if is_code {
            header = header.child(self.btn(
                "preview-wrap",
                if self.code_wrap { "No wrap" } else { "Wrap" },
                true,
                cx,
                |this, _, cx| this.toggle_code_wrap(cx),
            ));
        }
        if preview.content.copy_text().is_some() {
            header = header.child(self.btn("preview-copy", "Copy", true, cx, |this, _, cx| {
                this.copy_preview(cx)
            }));
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
            .h(gpui::relative(self.preview_ratio))
            .flex_none()
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(rgb(theme.border))
            .bg(rgb(theme.panel))
            .child(
                div()
                    .id("preview-splitter")
                    .h(px(5.))
                    .flex_none()
                    .w_full()
                    .cursor(gpui::CursorStyle::ResizeUpDown)
                    .bg(rgb(theme.border))
                    .hover(move |style| style.bg(rgb(theme.accent)))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                            this.begin_preview_drag(f32::from(event.position.y), window, cx);
                        }),
                    )
                    .on_click(cx.listener(|this, event: &gpui::ClickEvent, _, cx| {
                        if event.click_count() >= 2 {
                            this.reset_preview_ratio(cx);
                        }
                    })),
            )
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
        PreviewContent::Markdown { .. } => Some("Markdown"),
        PreviewContent::Text(text) => {
            let path = preview.path.as_deref().unwrap_or_else(|| Path::new(""));
            let syntax = Syntax::detect_with_text(path, text);
            (syntax != Syntax::Plain).then_some(syntax.label())
        }
        _ => None,
    }
}

fn image_view(path: &Path, zoom: f32) -> AnyElement {
    let dimensions = image::image_dimensions(path).ok();
    let content = if let Some((width, height)) = dimensions {
        img(path.to_path_buf())
            .w(px(width as f32 * zoom))
            .h(px(height as f32 * zoom))
            .object_fit(ObjectFit::Fill)
            .into_any_element()
    } else {
        img(path.to_path_buf())
            .w_full()
            .h_full()
            .object_fit(ObjectFit::Contain)
            .into_any_element()
    };
    div()
        .id("image-preview-scroll")
        .w_full()
        .h_full()
        .overflow_scroll()
        .child(content)
        .into_any_element()
}
