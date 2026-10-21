// Status bar matching WinDirStat with live progress and Pacman widget

use egui::{Color32, RichText, Ui};

use super::pacman::PacmanWidget;
use crate::model::item::Item;
use crate::scanner::stats::{ScanStats, ScanStatus};
use crate::utils::format::{format_number, format_size};

pub fn render_status_bar(
    ui: &mut Ui,
    stats: &mut ScanStats,
    selected_item: Option<&Item>,
    pacman: &mut PacmanWidget,
    action_status: Option<&str>,
) {
    ui.horizontal(|ui| {
        // Left section: Status or selected file info
        let is_scanning = stats.status == ScanStatus::Scanning;

        if let Some(msg) = action_status {
            ui.label(RichText::new(msg).color(Color32::from_rgb(255, 220, 100)));
        } else if is_scanning {
            let path_str = stats
                .current_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Initializing...".to_string());

            let display_path = if path_str.len() > 65 {
                format!("…{}", &path_str[path_str.len() - 64..])
            } else {
                path_str
            };

            ui.label(RichText::new(format!("Scanning: {}", display_path)).color(Color32::WHITE));
        } else if let Some(item) = selected_item {
            let type_str = if item.is_dir { "Directory" } else { "File" };
            ui.label(
                RichText::new(format!(
                    "{}: {} ({}) | Items: {}",
                    type_str,
                    item.path.display(),
                    format_size(item.size),
                    format_number(item.items as u64)
                ))
                .color(Color32::from_rgb(220, 225, 230)),
            );
        } else {
            ui.label(RichText::new("Ready").color(Color32::from_rgb(180, 180, 190)));
        }

        // Pacman widget when scanning
        if is_scanning {
            pacman.active = true;
            ui.add_space(15.0);
            pacman.show(ui, 140.0, 18.0);
        } else {
            pacman.active = false;
        }

        // Right-aligned stats
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if is_scanning || stats.files_scanned > 0 {
                let elapsed_secs = stats.elapsed.as_secs();
                let time_str = format!("{:02}:{:02}", elapsed_secs / 60, elapsed_secs % 60);

                if is_scanning {
                    ui.label(
                        RichText::new(format!(
                            "Time: {} | Speed: {:.0} files/s",
                            time_str,
                            stats.files_per_sec()
                        ))
                        .color(Color32::from_rgb(170, 170, 180)),
                    );
                } else {
                    ui.label(
                        RichText::new(format!("Elapsed: {}", time_str))
                            .color(Color32::from_rgb(160, 160, 170)),
                    );
                }

                ui.separator();

                ui.label(
                    RichText::new(format!(
                        "Files: {} | Dirs: {} | Total: {}",
                        format_number(stats.files_scanned as u64),
                        format_number(stats.dirs_scanned as u64),
                        format_size(stats.total_bytes)
                    ))
                    .color(Color32::from_rgb(200, 200, 210)),
                );
            }
        });
    });
}

