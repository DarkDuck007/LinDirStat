// Extension legend list view matching WinDirStat's top-right pane

use egui::{Color32, Rect, RichText, Sense, Stroke, Ui, Vec2};

use crate::model::extension::ExtensionInfo;
use crate::model::tree::TreeModel;
use crate::utils::format::{format_number, format_percent, format_size};

pub struct ExtListView {
    pub selected_extension: Option<String>,
}

impl Default for ExtListView {
    fn default() -> Self {
        Self {
            selected_extension: None,
        }
    }
}

impl ExtListView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        model: &TreeModel,
        selected_id: &mut Option<usize>,
    ) {
        if model.extensions.extensions.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("No extensions").color(Color32::from_rgb(120, 120, 130)));
            });
            return;
        }

        // Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("Extension").strong());
            ui.add_space(20.0);
            ui.label(RichText::new("Color").strong());
            ui.add_space(15.0);
            ui.label(RichText::new("Bytes (%)").strong());
            ui.add_space(35.0);
            ui.label(RichText::new("Bytes").strong());
            ui.add_space(20.0);
            ui.label(RichText::new("Files").strong());
        });

        ui.separator();

        let total_exts = model.extensions.extensions.len();
        let row_height = 20.0;
        ui.spacing_mut().item_spacing.y = 0.0;

        egui::ScrollArea::both()
            .id_salt("ext_list_scroll_area")
            .auto_shrink([false, false])
            .show_rows(ui, row_height, total_exts, |ui, row_range| {
                for idx in row_range {
                    if let Some(ext_info) = model.extensions.extensions.get(idx) {
                        self.render_ext_row(ui, ext_info, model, selected_id);
                    }
                }
            });
    }

    fn render_ext_row(
        &mut self,
        ui: &mut Ui,
        ext: &ExtensionInfo,
        model: &TreeModel,
        selected_id: &mut Option<usize>,
    ) {
        let is_selected = self.selected_extension.as_deref() == Some(&ext.extension);
        let row_height = 20.0;
        let (row_rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), row_height),
            Sense::click(),
        );

        if is_selected {
            ui.painter().rect_filled(row_rect, 2.0, Color32::from_rgb(45, 75, 120));
        } else if response.hovered() {
            ui.painter().rect_filled(row_rect, 2.0, Color32::from_rgb(35, 40, 50));
        }

        if response.clicked() {
            self.selected_extension = Some(ext.extension.clone());

            // Select largest file of this extension in model
            let mut largest_file_id = None;
            let mut max_sz = 0;
            for item in &model.items {
                if !item.is_dir && item.extension.as_deref() == Some(&ext.extension) {
                    if item.size >= max_sz {
                        max_sz = item.size;
                        largest_file_id = Some(item.id);
                    }
                }
            }
            if let Some(fid) = largest_file_id {
                *selected_id = Some(fid);
            }
        }

        let painter = ui.painter();
        let y = row_rect.center().y;
        let x_start = row_rect.min.x;

        // 1. Extension name
        painter.text(
            egui::pos2(x_start + 4.0, y),
            egui::Align2::LEFT_CENTER,
            &ext.extension,
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );

        // 2. Color swatch (WinDirStat style rectangle)
        let swatch_rect = Rect::from_min_max(
            egui::pos2(x_start + 85.0, y - 6.0),
            egui::pos2(x_start + 115.0, y + 6.0),
        );
        painter.rect_filled(swatch_rect, 1.0, ext.color);
        painter.rect_stroke(swatch_rect, 1.0, Stroke::new(1.0_f32, Color32::BLACK), egui::StrokeKind::Inside);

        // 3. Bytes %
        painter.text(
            egui::pos2(x_start + 140.0, y),
            egui::Align2::LEFT_CENTER,
            format_percent(ext.percentage),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(200, 200, 210),
        );

        // 4. Formatted Bytes
        painter.text(
            egui::pos2(x_start + 225.0, y),
            egui::Align2::LEFT_CENTER,
            format_size(ext.total_bytes),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(220, 220, 230),
        );

        // 5. Files Count
        painter.text(
            egui::pos2(x_start + 295.0, y),
            egui::Align2::LEFT_CENTER,
            format_number(ext.file_count as u64),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(180, 180, 190),
        );
    }
}
