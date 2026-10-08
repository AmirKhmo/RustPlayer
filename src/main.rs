#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
//! RustPlayer: PotPlayer-style media player. egui UI (Persian, RTL) + libmpv video via OpenGL.
mod app;
mod fa;
mod fonts;
mod mpv;
mod subs;
mod video;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let files: Vec<String> = std::env::args().skip(1).collect();
    let icon = egui::IconData {
        rgba: include_bytes!("../assets/icon_64.rgba").to_vec(),
        width: 64,
        height: 64,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RustPlayer")
            .with_icon(icon)
            .with_inner_size([1180.0, 720.0])
            .with_min_inner_size([520.0, 340.0])
            .with_drag_and_drop(true),
        renderer: eframe::Renderer::Glow,
        vsync: true,
        ..Default::default()
    };
    eframe::run_native(
        "RustPlayer",
        options,
        Box::new(move |cc| Box::new(app::App::new(cc, files))),
    )
}
