use gpui::prelude::*;
use gpui::{AnyElement, Context, div, px, rgb};

use crate::app::AppView;
use crate::checksum::ChecksumEntry;
use crate::fsops;
use crate::theme::Theme;

impl AppView {
    pub(crate) fn render_checksums_modal(
        &self,
        entries: &[ChecksumEntry],
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = self.theme;
        let mut list = div().flex().flex_col().gap_2();
        for entry in entries {
            list = list.child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .rounded(px(4.))
                    .border_1()
                    .border_color(rgb(theme.border))
                    .bg(rgb(theme.panel2))
                    .p_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.))
                                    .truncate()
                                    .text_sm()
                                    .text_color(rgb(theme.text))
                                    .child(entry.name.clone()),
                            )
                            .child(
                                div()
                                    .flex_none()
                                    .text_xs()
                                    .text_color(rgb(theme.muted))
                                    .child(fsops::human_size(entry.size)),
                            ),
                    )
                    .child(hash_row("SHA-256", &entry.sha256, theme))
                    .child(hash_row("SHA-1", &entry.sha1, theme))
                    .child(hash_row("MD5", &entry.md5, theme))
                    .child(hash_row("CRC32", &entry.crc32, theme)),
            );
        }
        let body = div()
            .id("checksums-body")
            .max_h(gpui::relative(0.62))
            .min_h(px(0.))
            .overflow_y_scroll()
            .child(list);
        let entries_copy = entries.to_vec();
        let entries_save = entries.to_vec();
        self.modal_panel(
            "Checksums",
            820.0,
            body,
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(self.btn(
                    "checksums-copy",
                    "Copy SHA256SUMS",
                    true,
                    cx,
                    move |this, _, cx| this.copy_checksums(&entries_copy, cx),
                ))
                .child(self.btn(
                    "checksums-save",
                    "Save SHA256SUMS",
                    true,
                    cx,
                    move |this, _, cx| this.save_checksums(&entries_save, cx),
                ))
                .child(
                    self.primary_btn("checksums-close", "Close", false, cx, |this, _, cx| {
                        this.close_modal(cx)
                    }),
                ),
            cx,
        )
    }

    pub(crate) fn copy_checksums(&mut self, entries: &[ChecksumEntry], cx: &mut Context<Self>) {
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(crate::checksum::manifest(
            entries,
        )));
        self.set_message("Copied SHA256SUMS", false);
        cx.notify();
    }
}

fn hash_row(label: &'static str, hash: &str, theme: Theme) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .w(px(64.))
                .flex_none()
                .text_xs()
                .text_color(rgb(theme.dim))
                .child(label),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.))
                .truncate()
                .font_family("monospace")
                .text_xs()
                .text_color(rgb(theme.muted))
                .child(hash.to_string()),
        )
        .into_any_element()
}
