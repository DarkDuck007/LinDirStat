// Directory tree list view matching WinDirStat's top-left pane

use egui::{Color32, Rect, RichText, Sense, Stroke, Ui, Vec2};

use crate::model::tree::TreeModel;
use crate::utils::format::{format_number, format_percent, format_size, format_timestamp};
use crate::utils::fs_ops;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Size,
    Allocated,
    Items,
    Files,
    Subdirs,
    LastChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct DirTreeView {
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub request_scroll_to_selected: bool,
}

impl Default for DirTreeView {
    fn default() -> Self {
        Self {
            sort_column: SortColumn::Size,
            sort_direction: SortDirection::Descending,
            request_scroll_to_selected: false,
        }
    }
}

impl DirTreeView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        model: &mut TreeModel,
        selected_id: &mut Option<usize>,
        action_status: &mut Option<String>,
        is_scanning: bool,
    ) {
        if model.is_empty() {
            ui.centered_and_justified(|ui| {
                if is_scanning {
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Spinner::new().size(24.0));
                        ui.add_space(8.0);
                        ui.label(RichText::new("Scanning directories...").strong().color(Color32::WHITE));
                        ui.label(RichText::new("The directory tree will populate once scanning completes.").color(Color32::from_rgb(160, 160, 170)));
                    });
                } else {
                    ui.label(RichText::new("No directory loaded. Click 'Open...' to scan a folder.").color(Color32::from_rgb(140, 140, 150)));
                }
            });
            return;
        }

        let total_root_size = model
            .root_id
            .and_then(|rid| model.get(rid))
            .map(|r| r.size)
            .unwrap_or(1);

        // Header row
        ui.horizontal(|ui| {
            self.header_button(ui, "Name", SortColumn::Name, 240.0);
            self.header_button(ui, "Subtree %", SortColumn::Size, 120.0);
            self.header_button(ui, "Size", SortColumn::Size, 85.0);
            self.header_button(ui, "Allocated", SortColumn::Allocated, 85.0);
            self.header_button(ui, "Items", SortColumn::Items, 65.0);
            self.header_button(ui, "Files", SortColumn::Files, 65.0);
            self.header_button(ui, "Subdirs", SortColumn::Subdirs, 65.0);
            self.header_button(ui, "Last Change", SortColumn::LastChange, 140.0);
        });

        ui.separator();

        let mut nodes_to_render = Vec::new();
        if let Some(root_id) = model.root_id {
            self.flatten_tree(model, root_id, 0, &mut nodes_to_render);
        }

        let total_nodes = nodes_to_render.len();
        let row_height = 20.0;
        let avail_h = ui.available_height();

        let mut scroll_area = egui::ScrollArea::both()
            .id_salt("dir_tree_scroll_area")
            .auto_shrink([false, false]);

        if self.request_scroll_to_selected {
            self.request_scroll_to_selected = false;
            if let Some(sel_id) = *selected_id {
                if let Some(row_idx) = nodes_to_render.iter().position(|&(id, _)| id == sel_id) {
                    let target_offset = (row_idx as f32 * row_height - avail_h / 2.0).max(0.0);
                    scroll_area = scroll_area.vertical_scroll_offset(target_offset);
                }
            }
        }

        ui.spacing_mut().item_spacing.y = 0.0;

        scroll_area.show_rows(ui, row_height, total_nodes, |ui, row_range| {
            for row_idx in row_range {
                if let Some(&(item_id, depth)) = nodes_to_render.get(row_idx) {
                    self.render_row(
                        ui,
                        model,
                        item_id,
                        depth,
                        total_root_size,
                        selected_id,
                        action_status,
                    );
                }
            }
        });
    }

    fn header_button(&mut self, ui: &mut Ui, title: &str, col: SortColumn, width: f32) {
        let is_selected = self.sort_column == col;
        let arrow = if is_selected {
            if self.sort_direction == SortDirection::Ascending { " ▲" } else { " ▼" }
        } else {
            ""
        };
        let label = format!("{}{}", title, arrow);

        let btn = egui::Button::new(RichText::new(label).strong())
            .min_size(Vec2::new(width, 20.0))
            .wrap_mode(egui::TextWrapMode::Truncate);

        if ui.add(btn).clicked() {
            if self.sort_column == col {
                self.sort_direction = match self.sort_direction {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                };
            } else {
                self.sort_column = col;
                self.sort_direction = SortDirection::Descending;
            }
        }
    }

    fn flatten_tree(
        &self,
        model: &TreeModel,
        node_id: usize,
        depth: usize,
        out: &mut Vec<(usize, usize)>,
    ) {
        out.push((node_id, depth));

        if let Some(node) = model.get(node_id) {
            // Expand root by default
            let is_expanded = node.expanded || node.parent.is_none();

            if is_expanded && node.is_dir {
                let mut children = node.children.clone();

                // Sort children
                let items_ref = &model.items;
                let col = self.sort_column;
                let dir = self.sort_direction;

                children.sort_by(|&a, &b| {
                    let ia = match items_ref.get(a) { Some(i) => i, None => return std::cmp::Ordering::Equal };
                    let ib = match items_ref.get(b) { Some(i) => i, None => return std::cmp::Ordering::Equal };

                    let ord = match col {
                        SortColumn::Name => ia.name.to_lowercase().cmp(&ib.name.to_lowercase()),
                        SortColumn::Size => ia.size.cmp(&ib.size),
                        SortColumn::Allocated => ia.allocated_size.cmp(&ib.allocated_size),
                        SortColumn::Items => ia.items.cmp(&ib.items),
                        SortColumn::Files => ia.files.cmp(&ib.files),
                        SortColumn::Subdirs => ia.subdirs.cmp(&ib.subdirs),
                        SortColumn::LastChange => ia.mtime.cmp(&ib.mtime),
                    };

                    if dir == SortDirection::Ascending {
                        ord
                    } else {
                        ord.reverse()
                    }
                });

                for child_id in children {
                    self.flatten_tree(model, child_id, depth + 1, out);
                }
            }
        }
    }

    fn render_row(
        &mut self,
        ui: &mut Ui,
        model: &mut TreeModel,
        item_id: usize,
        depth: usize,
        total_root_size: u64,
        selected_id: &mut Option<usize>,
        action_status: &mut Option<String>,
    ) {
        let is_selected = *selected_id == Some(item_id);

        let (name, is_dir, is_expanded, size, alloc, items_cnt, files_cnt, subdirs_cnt, mtime, has_children, path) = {
            let item = match model.get(item_id) {
                Some(i) => i,
                None => return,
            };
            (
                item.name.clone(),
                item.is_dir,
                item.expanded || item.parent.is_none(),
                item.size,
                item.allocated_size,
                item.items,
                item.files,
                item.subdirs,
                item.mtime,
                !item.children.is_empty(),
                item.path.clone(),
            )
        };

        let row_height = 20.0;
        let (row_rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width().max(840.0), row_height),
            Sense::click(),
        );

        // Highlight selection
        if is_selected {
            ui.painter().rect_filled(row_rect, 2.0, Color32::from_rgb(45, 75, 120));
        } else if response.hovered() {
            ui.painter().rect_filled(row_rect, 2.0, Color32::from_rgb(35, 40, 50));
        }

        let indent = (depth as f32) * 16.0;

        if response.clicked() {
            *selected_id = Some(item_id);
            if is_dir {
                if let Some(pos) = response.interact_pointer_pos() {
                    let rel_x = pos.x - row_rect.min.x;
                    if rel_x <= indent + 36.0 {
                        if let Some(item_mut) = model.get_mut(item_id) {
                            item_mut.expanded = !item_mut.expanded;
                        }
                    }
                }
            }
        }

        if response.double_clicked() && is_dir {
            if let Some(item_mut) = model.get_mut(item_id) {
                item_mut.expanded = !item_mut.expanded;
            }
        }

        // Context menu
        response.context_menu(|menu_ui| {
            menu_ui.label(format!("Item: {}", name));
            menu_ui.separator();
            if menu_ui.button("Open").clicked() {
                if let Err(e) = fs_ops::open_file(&path) {
                    *action_status = Some(e);
                }
                menu_ui.close_menu();
            }
            if menu_ui.button("Show in File Manager").clicked() {
                if let Err(e) = fs_ops::reveal_in_file_manager(&path) {
                    *action_status = Some(e);
                }
                menu_ui.close_menu();
            }
            if menu_ui.button("Open Terminal Here").clicked() {
                if let Err(e) = fs_ops::open_terminal(&path) {
                    *action_status = Some(e);
                }
                menu_ui.close_menu();
            }
            if menu_ui.button("Copy Path").clicked() {
                menu_ui.ctx().copy_text(path.display().to_string());
                *action_status = Some("Path copied".to_string());
                menu_ui.close_menu();
            }
            menu_ui.separator();
            if menu_ui.button("Move to Trash").clicked() {
                match fs_ops::move_to_trash(&path) {
                    Ok(_) => *action_status = Some(format!("Trashed {}", name)),
                    Err(e) => *action_status = Some(e),
                }
                menu_ui.close_menu();
            }
        });

        // Draw row cells manually for perfect tabular alignment
        let painter = ui.painter();
        let y = row_rect.center().y;

        // 1. Name column with indentation and expander
        let mut x = row_rect.min.x + indent + 4.0;

        if is_dir {
            let arrow = if has_children {
                if is_expanded { "▼" } else { "▶" }
            } else {
                " "
            };
            painter.text(
                egui::pos2(x, y),
                egui::Align2::LEFT_CENTER,
                arrow,
                egui::FontId::proportional(11.0),
                Color32::from_rgb(180, 180, 190),
            );
            x += 14.0;

            // Directory icon 📁
            painter.text(
                egui::pos2(x, y),
                egui::Align2::LEFT_CENTER,
                "📁",
                egui::FontId::proportional(13.0),
                Color32::from_rgb(240, 200, 80),
            );
            x += 18.0;
        } else {
            x += 14.0;
            // File icon 📄
            painter.text(
                egui::pos2(x, y),
                egui::Align2::LEFT_CENTER,
                "📄",
                egui::FontId::proportional(12.0),
                Color32::from_rgb(160, 180, 210),
            );
            x += 18.0;
        }

        // Name text (clipped to 240px from start)
        let name_col_end = row_rect.min.x + 240.0;
        let available_name_w = (name_col_end - x).max(20.0);
        let name_display = if (name.len() as f32 * 7.0) > available_name_w {
            let take = (available_name_w / 7.5).max(4.0) as usize;
            format!("{}…", &name.chars().take(take).collect::<String>())
        } else {
            name.clone()
        };

        painter.text(
            egui::pos2(x, y),
            egui::Align2::LEFT_CENTER,
            name_display,
            egui::FontId::proportional(13.0),
            if is_dir { Color32::WHITE } else { Color32::from_rgb(220, 220, 230) },
        );

        // 2. Subtree % Bar
        let pct_x = row_rect.min.x + 245.0;
        let pct_w = 110.0;
        let fraction = if total_root_size > 0 {
            (size as f32 / total_root_size as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let bar_rect = Rect::from_min_max(
            egui::pos2(pct_x, y - 5.0),
            egui::pos2(pct_x + pct_w, y + 5.0),
        );
        painter.rect_filled(bar_rect, 1.0, Color32::from_rgb(30, 30, 35));
        painter.rect_stroke(bar_rect, 1.0, Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 70)), egui::StrokeKind::Inside);

        if fraction > 0.001 {
            let fill_rect = Rect::from_min_max(
                bar_rect.min,
                egui::pos2(pct_x + pct_w * fraction, bar_rect.max.y),
            );
            painter.rect_filled(fill_rect, 1.0, Color32::from_rgb(50, 120, 220));
        }

        painter.text(
            egui::pos2(pct_x + pct_w / 2.0, y),
            egui::Align2::CENTER_CENTER,
            format_percent(fraction),
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );

        // 3. Size column (85px)
        let size_x = pct_x + pct_w + 10.0;
        painter.text(
            egui::pos2(size_x + 80.0, y),
            egui::Align2::RIGHT_CENTER,
            format_size(size),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(230, 230, 230),
        );

        // 4. Allocated column (85px)
        let alloc_x = size_x + 85.0;
        painter.text(
            egui::pos2(alloc_x + 80.0, y),
            egui::Align2::RIGHT_CENTER,
            format_size(alloc),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(170, 170, 180),
        );

        // 5. Items (65px)
        let items_x = alloc_x + 85.0;
        painter.text(
            egui::pos2(items_x + 60.0, y),
            egui::Align2::RIGHT_CENTER,
            format_number(items_cnt as u64),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(200, 200, 210),
        );

        // 6. Files (65px)
        let files_x = items_x + 65.0;
        painter.text(
            egui::pos2(files_x + 60.0, y),
            egui::Align2::RIGHT_CENTER,
            format_number(files_cnt as u64),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(180, 180, 190),
        );

        // 7. Subdirs (65px)
        let subdirs_x = files_x + 65.0;
        painter.text(
            egui::pos2(subdirs_x + 60.0, y),
            egui::Align2::RIGHT_CENTER,
            format_number(subdirs_cnt as u64),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(180, 180, 190),
        );

        // 8. Last Change (140px)
        let mtime_x = subdirs_x + 70.0;
        painter.text(
            egui::pos2(mtime_x, y),
            egui::Align2::LEFT_CENTER,
            format_timestamp(mtime),
            egui::FontId::proportional(11.0),
            Color32::from_rgb(150, 150, 160),
        );
    }
}
