use std::path::Path;

use gpui::prelude::*;
use gpui::{AnyElement, Context, HighlightStyle, ObjectFit, StyledText, div, img, px, rgb};

use crate::app::AppView;
use crate::hex::{self, HexPreview};
use crate::nfo;
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
            PreviewContent::Nfo(document) => nfo_view(
                document,
                theme,
                self.settings.nfo_font_size,
                self.settings.nfo_colors,
            ),
            PreviewContent::Hex(hex) => hex_view(hex, theme, self.settings.hex_colors),
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
        if matches!(preview.content, PreviewContent::Nfo(_)) {
            header = header.child(
                self.btn("preview-nfo-full", "Full", true, cx, |this, _, cx| {
                    this.open_nfo_full(cx)
                }),
            );
        }
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
        PreviewContent::Nfo(_) => Some("NFO"),
        PreviewContent::Hex(_) => Some("Hex"),
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

pub(crate) fn nfo_view(
    document: &nfo::NfoDocument,
    theme: Theme,
    font_size: f32,
    colors: bool,
) -> AnyElement {
    let highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = document
        .colors
        .iter()
        .map(|span| {
            (
                span.range.clone(),
                HighlightStyle {
                    color: Some(rgb(nfo::nfo_color(span.color)).into()),
                    font_weight: span.bold.then_some(gpui::FontWeight::BOLD),
                    ..Default::default()
                },
            )
        })
        .collect();
    let text = document.text.clone();
    let styled = if colors && !highlights.is_empty() {
        StyledText::new(text).with_highlights(highlights)
    } else {
        StyledText::new(text)
    };
    div()
        .id("nfo-scroll")
        .w_full()
        .h_full()
        .overflow_scroll()
        .bg(rgb(0x07090c))
        .border_1()
        .border_color(rgb(theme.border))
        .p_2()
        .child(
            div()
                .font_family("DejaVu Sans Mono")
                .text_size(px(font_size))
                .line_height(px(font_size + 3.0))
                .whitespace_nowrap()
                .text_color(rgb(nfo::nfo_color(7)))
                .child(styled),
        )
        .into_any_element()
}

fn hex_view(preview: &HexPreview, theme: Theme, colors: bool) -> AnyElement {
    let mut text = String::new();
    let mut highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = Vec::new();
    let header = hex::header();
    text.push_str(&header);
    text.push('\n');
    highlights.push((
        0..header.len(),
        HighlightStyle::color(rgb(theme.dim).into()),
    ));
    for (index, line) in preview.lines.iter().enumerate() {
        let line_start = text.len();
        text.push_str(&format!("{:08x}  ", line.offset));
        highlights.push((
            line_start..line_start + 8,
            HighlightStyle::color(rgb(theme.accent).into()),
        ));
        for byte_index in 0..16 {
            if let Some(byte) = line.bytes.get(byte_index) {
                let start = text.len();
                text.push_str(&format!("{byte:02x}"));
                highlights.push((
                    start..start + 2,
                    HighlightStyle::color(rgb(hex_byte_color(theme, *byte)).into()),
                ));
                text.push(' ');
            } else {
                text.push_str("   ");
            }
            if byte_index == 7 {
                text.push(' ');
            }
        }
        let separator_start = text.len();
        text.push_str(" |");
        highlights.push((
            separator_start..separator_start + 2,
            HighlightStyle::color(rgb(theme.border2).into()),
        ));
        for byte in &line.bytes {
            let start = text.len();
            let printable = (0x20..=0x7e).contains(byte);
            text.push(if printable { *byte as char } else { '.' });
            let color = if printable { theme.text } else { theme.dim };
            highlights.push((start..start + 1, HighlightStyle::color(rgb(color).into())));
        }
        for _ in line.bytes.len()..16 {
            text.push(' ');
        }
        text.push('|');
        if index + 1 < preview.lines.len() {
            text.push('\n');
        }
    }
    if preview.truncated {
        let start = text.len();
        text.push_str("\n[hex preview truncated]");
        highlights.push((
            start..text.len(),
            HighlightStyle::color(rgb(theme.amber).into()),
        ));
    }
    let styled = if colors {
        StyledText::new(text).with_highlights(highlights)
    } else {
        StyledText::new(preview.text())
    };
    div()
        .id("hex-scroll")
        .w_full()
        .h_full()
        .overflow_scroll()
        .bg(rgb(theme.input_bg))
        .border_1()
        .border_color(rgb(theme.border))
        .p_2()
        .child(
            div()
                .font_family("DejaVu Sans Mono")
                .text_size(px(12.5))
                .line_height(px(17.))
                .whitespace_nowrap()
                .text_color(rgb(theme.muted))
                .child(styled),
        )
        .into_any_element()
}

fn hex_byte_color(theme: Theme, byte: u8) -> u32 {
    match byte {
        0x00 => theme.dim,
        0x01..=0x1f => theme.purple,
        0x7f => theme.red,
        0x20..=0x7e => theme.text,
        _ => theme.amber,
    }
}
