use gpui::prelude::*;
use gpui::{AnyElement, Context, div, px, rgb};

use crate::app::AppView;

impl AppView {
    pub(crate) fn render_help_modal(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = self.theme;
        self.modal_panel_scroll(
            "7zrn - archive manager",
            720.0,
            {
                let rows: [(&str, &str); 38] = [
                    ("Enter", "Open folder / enter archive"),
                    ("Backspace", "Go to parent folder"),
                    ("Tab", "Switch focus between Files and Archive"),
                    ("Up / Down", "Move the cursor"),
                    ("Shift+Up / Down", "Extend the selection"),
                    ("Space", "Toggle selection, move down"),
                    ("Ctrl+Shift+T", "New tab"),
                    ("Ctrl+W", "Close tab"),
                    ("Ctrl+Tab", "Next tab"),
                    ("Ctrl+Shift+F", "Search current tree"),
                    ("Ctrl+Shift+P", "Command palette"),
                    ("Ctrl+A", "Select all entries"),
                    ("Ctrl+C", "Copy selected paths"),
                    ("Ctrl+F", "Filter the focused pane"),
                    ("Ctrl+L", "Edit the Files path"),
                    ("Ctrl+H", "Toggle hidden files"),
                    ("Ctrl+,", "Open settings"),
                    ("Ctrl+U", "Clear the focused input field"),
                    ("Ctrl+N", "New archive from selection"),
                    ("Ctrl+E", "Extract from the open archive"),
                    ("Ctrl+D", "Add Files selection to the archive"),
                    ("Ctrl+T", "Test the open archive"),
                    ("F2", "Rename archive entry"),
                    ("F7", "New archive folder"),
                    ("Delete", "Delete selected archive entries"),
                    ("F3", "Preview the selected file"),
                    ("Right click", "Open the pane context menu"),
                    ("F5", "Refresh the focused pane"),
                    ("F1", "This help"),
                    ("Esc", "Clear filter / close dialog"),
                    ("Alt+Left / Right", "Files history"),
                    ("Drag header", "Resize a column"),
                    ("Drag preview edge", "Resize the preview pane"),
                    (
                        "Drag files",
                        "Drop on Archive to add, drop on Files to extract",
                    ),
                    ("Drop files", "Drop external files on Files to reveal them"),
                    (
                        "Drag out",
                        "Not supported; use Extract and Browse to choose a folder",
                    ),
                    (
                        "Mouse",
                        "Click to select, double-click to open, Ctrl/Shift to multi-select",
                    ),
                    ("CLI", "7z for 7z/zip, GNU tar for tar.* (repacked on edit)"),
                ];
                let mut list = div().flex().flex_col().gap_1();
                for (keys, description) in rows {
                    list = list.child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(110.))
                                    .flex_none()
                                    .text_xs()
                                    .text_color(rgb(theme.accent))
                                    .child(keys),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.muted))
                                    .child(description),
                            ),
                    );
                }
                list
            },
            div().flex().items_center().gap_2().child(self.primary_btn(
                "help-close",
                "Close",
                false,
                cx,
                |this, _, cx| this.close_modal(cx),
            )),
            cx,
        )
    }
}
