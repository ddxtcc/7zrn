mod app;
mod archive;
mod checksum;
mod columns;
mod fsops;
mod hex;
mod job;
mod markdown;
mod nfo;
mod palette;
mod preview;
mod session;
mod settings;
mod state;
mod syntax;
mod text;
mod theme;
mod ui;

use std::path::PathBuf;

use app::AppView;
use gpui::prelude::*;
use gpui::{App, Application, Bounds, Focusable, WindowBounds, WindowOptions, px, size};

fn main() {
    Application::new().run(|cx: &mut App| {
        let cwd = std::env::current_dir().unwrap_or_else(|_| fsops::home_dir());
        let target = std::env::args_os().nth(1).map(PathBuf::from);
        let start = match &target {
            Some(target) if target.is_dir() => target.clone(),
            Some(target) => target
                .parent()
                .map(|parent| parent.to_path_buf())
                .unwrap_or_else(|| cwd.clone()),
            None => cwd,
        };
        let restore_session = target.is_none();
        let bounds = Bounds::centered(None, size(px(1220.0), px(780.0)), cx);
        let Ok(window) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("7zrn".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                app_id: Some("sevenzrn".to_string()),
                focus: true,
                show: true,
                is_resizable: true,
                window_min_size: Some(size(px(820.0), px(520.0))),
                ..Default::default()
            },
            move |_, cx| cx.new(|cx| AppView::new(start, restore_session, cx)),
        ) else {
            eprintln!("failed to open the 7zrn window");
            return;
        };
        let _ = window.update(cx, |view, window, cx| {
            window.focus(&view.focus_handle(cx));
            if let Some(target) = target {
                view.open_target(target, cx);
            }
        });
        cx.activate(true);
    });
}
