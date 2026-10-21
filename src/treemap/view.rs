// Interactive Treemap visualizer widget for egui

use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};

use super::cushion::{render_cushion_treemap, RenderOptions};
use super::layout::{layout_treemap, TreemapItem};
use crate::model::tree::TreeModel;
use crate::utils::format::format_size;
use crate::utils::fs_ops;

pub struct TreemapWidget {
    pub options: RenderOptions,
    pub treemap_items: Vec<TreemapItem>,
    pub texture: Option<egui::TextureHandle>,
    last_size: [usize; 2],
    last_root_id: Option<usize>,
    last_items_len: usize,
    pub hovered_id: Option<usize>,
    pub zoom_root_id: Option<usize>,
}

impl Default for TreemapWidget {
    fn default() -> Self {
        Self {
            options: RenderOptions::default(),
            treemap_items: Vec::new(),
            texture: None,
            last_size: [0, 0],
            last_root_id: None,
            last_items_len: 0,
            hovered_id: None,
            zoom_root_id: None,
        }
    }
}

impl TreemapWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invalidate(&mut self) {
        self.last_size = [0, 0];
        self.texture = None;
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        model: &TreeModel,
        selected_id: &mut Option<usize>,
        action_status: &mut Option<String>,
    ) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click());
        let width = rect.width().floor() as usize;
        let height = rect.height().floor() as usize;

        if width < 5 || height < 5 || model.is_empty() {
            ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(18, 18, 22));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No data to display. Open or scan a folder.",
                egui::FontId::proportional(14.0),
                Color32::from_rgb(120, 120, 130),
            );
            return;
        }

        let effective_root = self.zoom_root_id.or(model.root_id).unwrap_or(0);

        // Check if layout or texture needs recomputing
        let needs_layout = self.last_size != [width, height]
            || self.last_root_id != Some(effective_root)
            || self.last_items_len != model.len();

        if needs_layout {
            let local_rect = Rect::from_min_max(Pos2::ZERO, egui::pos2(width as f32, height as f32));
            self.treemap_items = layout_treemap(&model.items, effective_root, local_rect, self.options.config);
            self.last_size = [width, height];
            self.last_root_id = Some(effective_root);
            self.last_items_len = model.len();

            let color_image = render_cushion_treemap(
                &model.items,
                &self.treemap_items,
                &model.extensions,
                width,
                height,
                &self.options,
            );

            self.texture = Some(ui.ctx().load_texture(
                "treemap_texture",
                color_image,
                egui::TextureOptions::NEAREST,
            ));
        }

        // Draw rendered treemap texture
        if let Some(texture) = &self.texture {
            ui.painter().image(
                texture.id(),
                rect,
                Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        // Hit testing for hover / click
        self.hovered_id = None;
        if let Some(hover_pos) = response.hover_pos() {
            let rel_x = hover_pos.x - rect.min.x;
            let rel_y = hover_pos.y - rect.min.y;

            // Search smallest leaf containing cursor
            for tm in self.treemap_items.iter().rev() {
                if !tm.is_dir && tm.rect.contains(egui::pos2(rel_x, rel_y)) {
                    self.hovered_id = Some(tm.item_id);
                    break;
                }
            }

            // Fallback to directory if no leaf matched
            if self.hovered_id.is_none() {
                for tm in self.treemap_items.iter().rev() {
                    if tm.rect.contains(egui::pos2(rel_x, rel_y)) {
                        self.hovered_id = Some(tm.item_id);
                        break;
                    }
                }
            }
        }

        // Handle left click selection
        if response.clicked() {
            if let Some(hid) = self.hovered_id {
                *selected_id = Some(hid);
            }
        }

        // Handle right-click context menu
        response.context_menu(|menu_ui| {
            if let Some(sel_id) = *selected_id {
                if let Some(item) = model.get(sel_id) {
                    menu_ui.label(format!("File: {}", item.name));
                    menu_ui.separator();

                    if menu_ui.button("Open File").clicked() {
                        if let Err(e) = fs_ops::open_file(&item.path) {
                            *action_status = Some(e);
                        }
                        menu_ui.close_menu();
                    }

                    if menu_ui.button("Show in File Manager").clicked() {
                        if let Err(e) = fs_ops::reveal_in_file_manager(&item.path) {
                            *action_status = Some(e);
                        }
                        menu_ui.close_menu();
                    }

                    if menu_ui.button("Open Terminal Here").clicked() {
                        if let Err(e) = fs_ops::open_terminal(&item.path) {
                            *action_status = Some(e);
                        }
                        menu_ui.close_menu();
                    }

                    if menu_ui.button("Copy Path").clicked() {
                        menu_ui.ctx().copy_text(item.path.display().to_string());
                        *action_status = Some("Path copied to clipboard".to_string());
                        menu_ui.close_menu();
                    }

                    menu_ui.separator();

                    if item.is_dir {
                        if menu_ui.button("Zoom In (Make Root)").clicked() {
                            self.zoom_root_id = Some(sel_id);
                            self.invalidate();
                            menu_ui.close_menu();
                        }
                    }

                    if self.zoom_root_id.is_some() {
                        if menu_ui.button("Reset Zoom").clicked() {
                            self.zoom_root_id = None;
                            self.invalidate();
                            menu_ui.close_menu();
                        }
                    }

                    menu_ui.separator();

                    if menu_ui.button("Move to Trash").clicked() {
                        match fs_ops::move_to_trash(&item.path) {
                            Ok(_) => *action_status = Some(format!("Trashed {}", item.name)),
                            Err(e) => *action_status = Some(e),
                        }
                        menu_ui.close_menu();
                    }
                }
            } else {
                menu_ui.label("No item selected");
            }
        });

        // Highlight selected item (WinDirStat classic bright highlight outline)
        if let Some(sel_id) = *selected_id {
            for tm in &self.treemap_items {
                if tm.item_id == sel_id {
                    let screen_min = rect.min + Vec2::new(tm.rect.min.x, tm.rect.min.y);
                    let screen_max = rect.min + Vec2::new(tm.rect.max.x, tm.rect.max.y);
                    let sel_rect = Rect::from_min_max(screen_min, screen_max);

                    // WinDirStat bright double border
                    ui.painter().rect_stroke(
                        sel_rect,
                        0.0,
                        Stroke::new(2.5_f32, Color32::from_rgb(255, 255, 255)),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().rect_stroke(
                        sel_rect.expand(1.0),
                        0.0,
                        Stroke::new(1.0_f32, Color32::from_rgb(0, 0, 0)),
                        egui::StrokeKind::Inside,
                    );
                    break;
                }
            }
        }

        // Tooltip on hover
        if let Some(hid) = self.hovered_id {
            if let Some(item) = model.get(hid) {
                response.on_hover_ui_at_pointer(|tip_ui| {
                    tip_ui.set_max_width(450.0);
                    tip_ui.heading(&item.name);
                    tip_ui.horizontal(|h| {
                        h.label("Size:");
                        h.strong(format_size(item.size));
                    });
                    tip_ui.horizontal(|h| {
                        h.label("Path:");
                        h.monospace(item.path.display().to_string());
                    });
                    if let Some(ext) = &item.extension {
                        tip_ui.horizontal(|h| {
                            h.label("Type:");
                            h.label(ext);
                        });
                    }
                });
            }
        }
    }
}
