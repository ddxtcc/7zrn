use gpui::prelude::*;
use gpui::{AnyElement, Context, div, px};

use crate::app::AppView;
use crate::nfo::NfoDocument;
use crate::ui::preview::nfo_view;

impl AppView {
    pub(crate) fn render_nfo_modal(
        &self,
        title: &str,
        document: &NfoDocument,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let document_copy = document.clone();
        let body = div().h(gpui::relative(0.78)).min_h(px(0.)).child(nfo_view(
            document,
            self.theme,
            self.settings.nfo_font_size,
            self.settings.nfo_colors,
        ));
        self.modal_panel(
            title,
            1000.0,
            body,
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(self.btn("nfo-copy", "Copy", true, cx, move |this, _, cx| {
                    this.copy_nfo(&document_copy, cx)
                }))
                .child(
                    self.primary_btn("nfo-close", "Close", false, cx, |this, _, cx| {
                        this.close_modal(cx)
                    }),
                ),
            cx,
        )
    }

    pub(crate) fn copy_nfo(&mut self, document: &NfoDocument, cx: &mut Context<Self>) {
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(document.text.clone()));
        self.set_message("Copied NFO text", false);
        cx.notify();
    }
}
