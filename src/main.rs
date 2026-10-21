// LinDirStat - WinDirStat clone for Linux written in Rust using egui

use eframe::egui;
use lindirstat::LinDirStatApp;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    // Optional path from command line arguments
    let target_dir: Option<PathBuf> = std::env::args().nth(1).map(PathBuf::from);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LinDirStat")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([700.0, 450.0]),
        ..Default::default()
    };

    eframe::run_native(
        "LinDirStat",
        native_options,
        Box::new(move |_cc| {
            let app = LinDirStatApp::new_with_path(target_dir);
            Ok(Box::new(app))
        }),
    )
}
