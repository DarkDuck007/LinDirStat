// WinDirStat-style toolbar with action buttons and toggles

use egui::{RichText, Ui};

pub enum ToolbarAction {
    OpenDirectory,
    Rescan,
    StopScan,
    ToggleCushion,
    ToggleGrid,
    ZoomIn,
    ZoomOut,
    RevealInFileManager,
    OpenTerminal,
    TrashSelected,
}

pub fn render_toolbar(
    ui: &mut Ui,
    is_scanning: bool,
    cushion_enabled: bool,
    grid_enabled: bool,
    has_selection: bool,
    can_zoom_out: bool,
) -> Option<ToolbarAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        if ui.button(RichText::new("📂 Open...").strong()).clicked() {
            action = Some(ToolbarAction::OpenDirectory);
        }

        ui.add_enabled_ui(!is_scanning, |ui| {
            if ui.button(RichText::new("🔄 Refresh")).clicked() {
                action = Some(ToolbarAction::Rescan);
            }
        });

        if is_scanning {
            if ui.button(RichText::new("⏹ Stop").color(egui::Color32::from_rgb(255, 100, 100))).clicked() {
                action = Some(ToolbarAction::StopScan);
            }
        }

        ui.separator();

        let cushion_label = if cushion_enabled { "🛋 Cushion: ON" } else { "🛋 Cushion: OFF" };
        if ui.selectable_label(cushion_enabled, cushion_label).clicked() {
            action = Some(ToolbarAction::ToggleCushion);
        }

        let grid_label = if grid_enabled { "▦ Grid: ON" } else { "▦ Grid: OFF" };
        if ui.selectable_label(grid_enabled, grid_label).clicked() {
            action = Some(ToolbarAction::ToggleGrid);
        }

        ui.separator();

        ui.add_enabled_ui(has_selection, |ui| {
            if ui.button("🔍 Zoom In").clicked() {
                action = Some(ToolbarAction::ZoomIn);
            }
        });

        ui.add_enabled_ui(can_zoom_out, |ui| {
            if ui.button("🔍- Zoom Out").clicked() {
                action = Some(ToolbarAction::ZoomOut);
            }
        });

        ui.separator();

        ui.add_enabled_ui(has_selection, |ui| {
            if ui.button("📁 Reveal").clicked() {
                action = Some(ToolbarAction::RevealInFileManager);
            }
            if ui.button("💻 Terminal").clicked() {
                action = Some(ToolbarAction::OpenTerminal);
            }
            if ui.button("🗑 Trash").clicked() {
                action = Some(ToolbarAction::TrashSelected);
            }
        });
    });

    action
}

