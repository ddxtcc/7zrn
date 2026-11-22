use gpui::prelude::*;
use gpui::{Context, Render, Window, div, px, rgb};

use crate::theme::Theme;

pub(crate) struct DragChip {
    pub(crate) label: String,
    pub(crate) theme: Theme,
}

impl Render for DragChip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_3()
            .py_1()
            .rounded(px(4.))
            .bg(rgb(self.theme.panel2))
            .border_1()
            .border_color(rgb(self.theme.border2))
            .text_xs()
            .text_color(rgb(self.theme.text))
            .child(self.label.clone())
    }
}
