use gpui::prelude::*;
use gpui::{AnyElement, ClipboardItem, Context, div, px, rgb};

use crate::app::AppView;
use crate::archive::ArchiveStats;
use crate::fsops;
use crate::state::Modal;
use crate::theme::Theme;

impl AppView {
    pub(crate) fn render_archive_properties_modal(
        &self,
        stats: &ArchiveStats,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = self.theme;
        let stats_copy = stats.clone();
        self.modal_panel_scroll(
            "Archive properties",
            760.0,
            stats_body(stats, theme),
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    self.btn("stats-copy", "Copy", true, cx, move |this, _, cx| {
                        this.copy_stats(&stats_copy, cx)
                    }),
                )
                .child(
                    self.primary_btn("stats-close", "Close", false, cx, |this, _, cx| {
                        this.close_modal(cx)
                    }),
                ),
            cx,
        )
    }

    pub(crate) fn render_test_passed_modal(
        &self,
        stats: &ArchiveStats,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = self.theme;
        let body = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(theme.muted))
                    .child("No errors detected."),
            )
            .child(stats_body(stats, theme));
        let stats_copy = stats.clone();
        let stats_details = stats.clone();
        self.modal_panel_scroll(
            "Archive test passed",
            760.0,
            body,
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(self.btn("test-copy", "Copy", true, cx, move |this, _, cx| {
                    this.copy_stats(&stats_copy, cx)
                }))
                .child(self.btn(
                    "test-properties",
                    "Details",
                    true,
                    cx,
                    move |this, _, cx| {
                        this.modal = Some(Modal::ArchiveProperties {
                            stats: stats_details.clone(),
                        });
                        cx.notify();
                    },
                ))
                .child(
                    self.primary_btn("test-close", "Close", false, cx, |this, _, cx| {
                        this.close_modal(cx)
                    }),
                ),
            cx,
        )
    }

    pub(crate) fn copy_stats(&mut self, stats: &ArchiveStats, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(stats.summary()));
        self.set_message("Copied archive details", false);
        cx.notify();
    }
}

fn stats_body(stats: &ArchiveStats, theme: Theme) -> AnyElement {
    let body = div()
        .flex()
        .flex_col()
        .gap_3()
        .child(summary_rows(stats, theme))
        .child(compression_graph(stats, theme))
        .child(category_graph(stats, theme))
        .child(largest_files(stats, theme));
    div()
        .id("stats-body")
        .max_h(gpui::relative(0.62))
        .min_h(px(0.))
        .overflow_y_scroll()
        .child(body)
        .into_any_element()
}

fn summary_rows(stats: &ArchiveStats, theme: Theme) -> AnyElement {
    let rows = [
        ("Path", stats.path.display().to_string()),
        ("Format", stats.format.label().to_string()),
        ("Archive size", fsops::human_size(stats.archive_size)),
        ("Uncompressed", fsops::human_size(stats.total_size)),
        ("Saved", fsops::human_size(stats.saved_bytes())),
        (
            "Compression ratio",
            format!("{:.1}%", stats.compression_ratio()),
        ),
        ("Files", stats.file_count.to_string()),
        ("Folders", stats.folder_count.to_string()),
        ("Encrypted entries", stats.encrypted_count.to_string()),
        ("Methods", stats.methods.clone()),
    ];
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .children(rows.into_iter().map(|(label, value)| {
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .w(px(130.))
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
                        .text_xs()
                        .text_color(rgb(theme.text))
                        .child(value),
                )
        }))
        .into_any_element()
}

fn compression_graph(stats: &ArchiveStats, theme: Theme) -> AnyElement {
    let ratio = stats.compression_ratio();
    let total = stats.total_size.max(1);
    let filled = (stats.archive_size as f64 / total as f64).clamp(0.0, 1.0) as f32;
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(theme.dim))
                        .child("Compression"),
                )
                .child(div().text_xs().text_color(rgb(theme.muted)).child(format!(
                    "{} / {} ({:.1}%)",
                    fsops::human_size(stats.archive_size),
                    fsops::human_size(stats.total_size),
                    ratio
                ))),
        )
        .child(
            div()
                .h(px(14.))
                .w_full()
                .rounded(px(3.))
                .bg(rgb(theme.border))
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .w(gpui::relative(filled))
                        .bg(rgb(theme.accent)),
                ),
        )
        .child(div().text_xs().text_color(rgb(theme.dim)).child(format!(
            "Saved {} of uncompressed data",
            fsops::human_size(stats.saved_bytes())
        )))
        .into_any_element()
}

fn category_graph(stats: &ArchiveStats, theme: Theme) -> AnyElement {
    let mut bar = div()
        .h(px(20.))
        .w_full()
        .flex()
        .rounded(px(4.))
        .overflow_hidden()
        .bg(rgb(theme.border));
    for (index, category) in stats.categories.iter().enumerate() {
        bar = bar.child(
            div()
                .h_full()
                .w(gpui::relative((category.share / 100.0) as f32))
                .bg(rgb(category_color(theme, index))),
        );
    }
    let legend =
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_1()
            .children(
                stats
                    .categories
                    .iter()
                    .enumerate()
                    .map(|(index, category)| {
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .size(px(8.))
                                    .flex_none()
                                    .rounded(px(2.))
                                    .bg(rgb(category_color(theme, index))),
                            )
                            .child(
                                div()
                                    .w(px(90.))
                                    .flex_none()
                                    .text_xs()
                                    .text_color(rgb(theme.muted))
                                    .child(category.label),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_xs()
                                    .text_color(rgb(theme.text))
                                    .child(fsops::human_size(category.bytes)),
                            )
                            .child(
                                div()
                                    .w(px(52.))
                                    .text_right()
                                    .text_xs()
                                    .text_color(rgb(theme.dim))
                                    .child(format!("{:.1}%", category.share)),
                            )
                    }),
            );
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_xs()
                .text_color(rgb(theme.dim))
                .child("Size by type"),
        )
        .child(bar)
        .child(legend)
        .into_any_element()
}

fn largest_files(stats: &ArchiveStats, theme: Theme) -> AnyElement {
    let largest = &stats.largest;
    let rows = div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .children(largest.iter().map(|entry| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .truncate()
                        .text_xs()
                        .text_color(rgb(theme.text))
                        .child(entry.name.clone()),
                )
                .child(
                    div()
                        .w(px(110.))
                        .h(px(7.))
                        .flex_none()
                        .rounded(px(3.))
                        .bg(rgb(theme.border))
                        .overflow_hidden()
                        .child(
                            div()
                                .h_full()
                                .w(gpui::relative((entry.share / 100.0) as f32))
                                .bg(rgb(theme.amber)),
                        ),
                )
                .child(
                    div()
                        .w(px(72.))
                        .flex_none()
                        .text_right()
                        .text_xs()
                        .text_color(rgb(theme.muted))
                        .child(fsops::human_size(entry.bytes)),
                )
                .child(
                    div()
                        .w(px(48.))
                        .flex_none()
                        .text_right()
                        .text_xs()
                        .text_color(rgb(theme.dim))
                        .child(format!("{:.1}%", entry.share)),
                )
        }));
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(rgb(theme.dim))
                .child("Largest files"),
        )
        .child(rows)
        .into_any_element()
}

fn category_color(theme: Theme, index: usize) -> u32 {
    match index {
        0 => theme.accent,
        1 => theme.amber,
        2 => theme.purple,
        3 => theme.red,
        4 => theme.muted,
        5 => theme.dim,
        _ => theme.border2,
    }
}
