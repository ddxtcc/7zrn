use gpui::prelude::*;
use gpui::{
    AnyElement, ClickEvent, Context, CursorStyle, IntoElement, MouseButton, MouseDownEvent,
    SharedString, Window, div, px, rgb,
};

use crate::app::AppView;
use crate::state::EditTarget;
use crate::text::clamp_boundary;

pub(crate) const ROW_HEIGHT: f32 = 27.0;

pub(crate) struct InputField<'a> {
    pub(crate) id: &'static str,
    pub(crate) text: &'a str,
    pub(crate) placeholder: &'a str,
    pub(crate) active: bool,
    pub(crate) cursor: usize,
    pub(crate) width: Option<f32>,
    pub(crate) target: Option<EditTarget>,
    pub(crate) click_text: String,
}

impl AppView {
    pub(crate) fn empty_state(
        &self,
        title: &str,
        subtitle: &str,
        extra: Vec<AnyElement>,
        error: bool,
    ) -> AnyElement {
        let theme = self.theme;
        div()
            .flex_1()
            .min_h(px(0.))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .px_6()
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(if error { theme.red } else { theme.muted }))
                    .child(title.to_string()),
            )
            .child(
                div()
                    .max_w(px(460.))
                    .text_center()
                    .text_xs()
                    .text_color(rgb(theme.dim))
                    .child(subtitle.to_string()),
            )
            .children(extra)
            .into_any_element()
    }

    pub(crate) fn modal_panel(
        &self,
        title: &str,
        width: f32,
        body: impl IntoElement,
        buttons: impl IntoElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.modal_panel_with(title, width, body, buttons, false, cx)
    }

    pub(crate) fn modal_panel_scroll(
        &self,
        title: &str,
        width: f32,
        body: impl IntoElement,
        buttons: impl IntoElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.modal_panel_with(title, width, body, buttons, true, cx)
    }

    fn modal_panel_with(
        &self,
        title: &str,
        width: f32,
        body: impl IntoElement,
        buttons: impl IntoElement,
        scroll: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = self.theme;
        let mut panel = div()
            .w(px(width))
            .max_w(gpui::relative(0.92))
            .max_h(gpui::relative(0.9))
            .overflow_hidden()
            .flex()
            .flex_col()
            .bg(rgb(theme.panel))
            .border_1()
            .border_color(rgb(theme.border2))
            .rounded(px(8.))
            .shadow_lg()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                div()
                    .h(px(40.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .border_b_1()
                    .border_color(rgb(theme.border))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(theme.text))
                            .child(title.to_string()),
                    )
                    .child(div().flex_1())
                    .child(self.btn("modal-close", "×", true, cx, |this, _, cx| {
                        this.close_modal(cx)
                    })),
            );
        if scroll {
            panel = panel.h(gpui::relative(0.9));
        }
        let body = if scroll {
            div()
                .id("modal-scroll-body")
                .p_4()
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scroll()
                .child(body)
                .into_any_element()
        } else {
            div().p_4().child(body).into_any_element()
        };
        panel
            .child(body)
            .child(
                div()
                    .p_3()
                    .flex_none()
                    .border_t_1()
                    .border_color(rgb(theme.border))
                    .flex()
                    .items_center()
                    .justify_end()
                    .child(buttons),
            )
            .into_any_element()
    }

    pub(crate) fn btn<F>(
        &self,
        id: &'static str,
        label: impl Into<SharedString>,
        enabled: bool,
        cx: &mut Context<Self>,
        action: F,
    ) -> AnyElement
    where
        F: Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    {
        let theme = self.theme;
        let label: SharedString = label.into();
        let mut button = div()
            .id(id)
            .h(px(22.))
            .px_2()
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(4.))
            .border_1()
            .border_color(rgb(if enabled { theme.border } else { theme.panel }))
            .text_xs()
            .text_color(rgb(if enabled { theme.text } else { theme.dim }))
            .child(label);
        if enabled {
            button = button
                .cursor_pointer()
                .hover(move |style| style.bg(rgb(theme.hover)).border_color(rgb(theme.border2)))
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    action(this, window, cx);
                }));
        }
        button.into_any_element()
    }

    pub(crate) fn primary_btn<F>(
        &self,
        id: &'static str,
        label: &'static str,
        danger: bool,
        cx: &mut Context<Self>,
        action: F,
    ) -> AnyElement
    where
        F: Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    {
        let theme = self.theme;
        div()
            .id(id)
            .h(px(26.))
            .px_3()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(4.))
            .bg(rgb(if danger { theme.red } else { theme.accent }))
            .text_color(rgb(if danger {
                theme.danger_text
            } else {
                theme.accent_text
            }))
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .cursor_pointer()
            .hover(|style| style.opacity(0.9))
            .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                action(this, window, cx);
            }))
            .child(label)
            .into_any_element()
    }

    pub(crate) fn chip<F>(
        &self,
        id: &'static str,
        label: &str,
        selected: bool,
        cx: &mut Context<Self>,
        action: F,
    ) -> AnyElement
    where
        F: Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    {
        let theme = self.theme;
        div()
            .id(id)
            .h(px(24.))
            .px_2()
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(4.))
            .border_1()
            .border_color(rgb(if selected { theme.accent } else { theme.border }))
            .bg(rgb(if selected {
                theme.accent_dim
            } else {
                theme.panel2
            }))
            .text_xs()
            .text_color(rgb(if selected { theme.accent } else { theme.muted }))
            .cursor_pointer()
            .hover(move |style| style.border_color(rgb(theme.border2)).bg(rgb(theme.hover)))
            .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                action(this, window, cx);
            }))
            .child(label.to_string())
            .into_any_element()
    }

    pub(crate) fn input_field(&self, input: InputField<'_>, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        let InputField {
            id,
            text,
            placeholder,
            active,
            cursor,
            width,
            target,
            click_text,
        } = input;
        let mut field = div()
            .id(id)
            .h(px(26.))
            .flex()
            .items_center()
            .px_2()
            .rounded(px(4.))
            .bg(rgb(theme.input_bg))
            .border_1()
            .border_color(rgb(if active { theme.accent } else { theme.border }))
            .overflow_hidden();
        field = match width {
            Some(width) => field.w(px(width)).flex_none(),
            None => field.flex_1().min_w(px(0.)),
        };
        if let Some(target) = target {
            field = field.cursor(CursorStyle::IBeam).on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                    this.begin_edit(target, click_text.clone(), window, cx);
                }),
            );
        }
        let masked = matches!(
            target,
            Some(EditTarget::Password | EditTarget::NewPassword | EditTarget::ConvertPassword)
        );
        if active {
            let cursor = clamp_boundary(text, cursor);
            let (before, after) = if masked {
                (
                    "•".repeat(text[..cursor].chars().count()),
                    "•".repeat(text[cursor..].chars().count()),
                )
            } else {
                (text[..cursor].to_string(), text[cursor..].to_string())
            };
            field = field.child(
                div()
                    .flex()
                    .items_center()
                    .min_w(px(0.))
                    .child(div().whitespace_nowrap().child(before))
                    .child(
                        div()
                            .w(px(1.5))
                            .h(px(14.))
                            .flex_none()
                            .bg(rgb(theme.accent)),
                    )
                    .child(div().whitespace_nowrap().child(after)),
            );
            if text.is_empty() {
                field = field.child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.dim))
                        .child(placeholder.to_string()),
                );
            }
        } else {
            let display = if text.is_empty() {
                placeholder.to_string()
            } else if masked {
                "•".repeat(text.chars().count())
            } else {
                text.to_string()
            };
            field = field.child(
                div()
                    .truncate()
                    .text_color(rgb(if text.is_empty() {
                        theme.dim
                    } else {
                        theme.text
                    }))
                    .child(display),
            );
        }
        field.into_any_element()
    }
}
