use gpui::prelude::*;
use gpui::{Context, MouseButton, div, px, rgb};

use crate::app::AppView;

impl AppView {
    pub(crate) fn render_tab_bar(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = self.theme;
        let active_title = self.active_tab_title();
        let mut bar = div()
            .h(px(28.))
            .flex_none()
            .flex()
            .items_center()
            .gap_1()
            .px_2()
            .bg(rgb(theme.panel))
            .border_b_1()
            .border_color(rgb(theme.border));
        for (index, tab) in self.tabs.iter().enumerate() {
            let active = index == self.active_tab;
            let title = if active {
                active_title.clone()
            } else {
                tab.title.clone()
            };
            let can_close = self.tabs.len() > 1;
            bar = bar.child(
                div()
                    .id(("tab", index))
                    .h(px(22.))
                    .max_w(px(190.))
                    .min_w(px(80.))
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .rounded(px(4.))
                    .cursor_pointer()
                    .bg(rgb(if active { theme.panel2 } else { theme.bg }))
                    .border_1()
                    .border_color(rgb(if active { theme.border2 } else { theme.border }))
                    .hover(move |style| style.bg(rgb(theme.hover)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.switch_tab(index, cx);
                    }))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.))
                            .truncate()
                            .text_xs()
                            .text_color(rgb(if active { theme.text } else { theme.muted }))
                            .child(title),
                    )
                    .when(can_close, |tab| {
                        tab.child(
                            div()
                                .id(("tab-close", index))
                                .px_1()
                                .flex_none()
                                .text_xs()
                                .text_color(rgb(theme.dim))
                                .hover(move |style| {
                                    style.text_color(rgb(theme.red)).bg(rgb(theme.hover))
                                })
                                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    cx.stop_propagation();
                                    this.close_tab(index, cx);
                                }))
                                .child("×"),
                        )
                    }),
            );
        }
        bar.child(self.btn("tab-new", "+", true, cx, |this, _, cx| this.new_tab(cx)))
            .into_any_element()
    }
}
