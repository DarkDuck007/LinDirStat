// LinDirStat Main Application State and GUI layout

use crossbeam_channel::Receiver;
use eframe::egui;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::model::tree::TreeModel;
use crate::scanner::stats::{ScanStats, ScanStatus};
use crate::scanner::worker::{ScanMessage, ScannerOptions, ScannerWorker};
use crate::treemap::view::TreemapWidget;
use crate::utils::fs_ops;
use crate::views::dir_tree::DirTreeView;
use crate::views::ext_list::ExtListView;
use crate::views::open_dialog::OpenDialog;
use crate::views::pacman::PacmanWidget;
use crate::views::status_bar::render_status_bar;
use crate::views::toolbar::{render_toolbar, ToolbarAction};

pub struct LinDirStatApp {
    pub model: TreeModel,
    pub selected_id: Option<usize>,
    pub last_selected_id: Option<usize>,
    pub stats: ScanStats,
    pub options: ScannerOptions,

    // Views
    pub dir_tree_view: DirTreeView,
    pub ext_list_view: ExtListView,
    pub treemap_widget: TreemapWidget,
    pub pacman: PacmanWidget,
    pub open_dialog: OpenDialog,

    // Layout splits
    pub top_bottom_split: f32, // Height fraction for top panes (default 0.50)
    pub left_right_split: f32, // Width fraction for dir tree vs ext list (default 0.65)

    // Communication with scanner worker
    scan_rx: Option<Receiver<ScanMessage>>,
    cancel_flag: Arc<AtomicBool>,

    // Action feedback
    pub action_status: Option<String>,
    pub show_about_dialog: bool,
}

impl Default for LinDirStatApp {
    fn default() -> Self {
        Self {
            model: TreeModel::new(),
            selected_id: None,
            last_selected_id: None,
            stats: ScanStats::new(),
            options: ScannerOptions::default(),
            dir_tree_view: DirTreeView::new(),
            ext_list_view: ExtListView::new(),
            treemap_widget: TreemapWidget::new(),
            pacman: PacmanWidget::new(false),
            open_dialog: OpenDialog::new(),
            top_bottom_split: 0.50,
            left_right_split: 0.65,
            scan_rx: None,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            action_status: None,
            show_about_dialog: false,
        }
    }
}

impl LinDirStatApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    pub fn new_with_path(target_dir: Option<PathBuf>) -> Self {
        let mut app = Self::default();
        if let Some(path) = target_dir {
            if path.exists() {
                app.start_scan(path);
            } else {
                app.open_dialog.open(None);
            }
        } else {
            // Open drive/folder selector on startup like WinDirStat
            app.open_dialog.open(None);
        }
        app
    }

    pub fn start_scan(&mut self, path: PathBuf) {
        // Cancel existing scan if any
        self.cancel_scan();

        self.stats.start(path.clone());
        self.selected_id = None;
        self.action_status = None;
        self.treemap_widget.zoom_root_id = None;
        self.cancel_flag = Arc::new(AtomicBool::new(false));

        let (tx, rx) = crossbeam_channel::unbounded();
        self.scan_rx = Some(rx);

        ScannerWorker::scan_async(
            path,
            self.options.clone(),
            Arc::clone(&self.cancel_flag),
            tx,
        );
    }

    pub fn cancel_scan(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        self.stats.status = ScanStatus::Canceled;
        self.pacman.active = false;
    }

    fn check_scanner_messages(&mut self) {
        let mut close_channel = false;
        if let Some(rx) = &self.scan_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    ScanMessage::Progress {
                        current_path,
                        files_count,
                        dirs_count,
                        bytes_count,
                    } => {
                        self.stats.current_path = Some(current_path);
                        self.stats.files_scanned = files_count;
                        self.stats.dirs_scanned = dirs_count;
                        self.stats.total_bytes = bytes_count;
                        self.stats.update_elapsed();
                    }
                    ScanMessage::Finished(tree) => {
                        self.stats.status = ScanStatus::Finished;
                        self.stats.files_scanned = tree.root_id.map(|r| tree.items[r].files).unwrap_or(0);
                        self.stats.dirs_scanned = tree.root_id.map(|r| tree.items[r].subdirs).unwrap_or(0);
                        self.stats.total_bytes = tree.root_id.map(|r| tree.items[r].size).unwrap_or(0);
                        self.stats.current_path = None;
                        self.stats.update_elapsed();
                        self.model = tree;
                        self.treemap_widget.invalidate();
                        close_channel = true;
                    }
                    ScanMessage::Canceled => {
                        self.stats.status = ScanStatus::Canceled;
                        self.stats.current_path = None;
                        close_channel = true;
                    }
                    ScanMessage::Error(err) => {
                        self.stats.status = ScanStatus::Error(err.clone());
                        self.action_status = Some(err);
                        close_channel = true;
                    }
                }
            }
        }
        if close_channel {
            self.scan_rx = None;
        }
    }

    fn handle_toolbar_action(&mut self, action: ToolbarAction) {
        match action {
            ToolbarAction::OpenDirectory => {
                self.open_dialog.open(self.stats.root_path.clone());
            }
            ToolbarAction::Rescan => {
                if let Some(root) = self.stats.root_path.clone() {
                    self.start_scan(root);
                }
            }
            ToolbarAction::StopScan => {
                self.cancel_scan();
            }
            ToolbarAction::ToggleCushion => {
                self.treemap_widget.options.cushion = !self.treemap_widget.options.cushion;
                self.treemap_widget.invalidate();
            }
            ToolbarAction::ToggleGrid => {
                self.treemap_widget.options.grid = !self.treemap_widget.options.grid;
                self.treemap_widget.invalidate();
            }
            ToolbarAction::ZoomIn => {
                if let Some(sel_id) = self.selected_id {
                    if let Some(item) = self.model.get(sel_id) {
                        if item.is_dir {
                            self.treemap_widget.zoom_root_id = Some(sel_id);
                            self.treemap_widget.invalidate();
                        }
                    }
                }
            }
            ToolbarAction::ZoomOut => {
                self.treemap_widget.zoom_root_id = None;
                self.treemap_widget.invalidate();
            }
            ToolbarAction::RevealInFileManager => {
                if let Some(sel_id) = self.selected_id {
                    if let Some(item) = self.model.get(sel_id) {
                        if let Err(e) = fs_ops::reveal_in_file_manager(&item.path) {
                            self.action_status = Some(e);
                        }
                    }
                }
            }
            ToolbarAction::OpenTerminal => {
                if let Some(sel_id) = self.selected_id {
                    if let Some(item) = self.model.get(sel_id) {
                        if let Err(e) = fs_ops::open_terminal(&item.path) {
                            self.action_status = Some(e);
                        }
                    }
                }
            }
            ToolbarAction::TrashSelected => {
                if let Some(sel_id) = self.selected_id {
                    if let Some(item) = self.model.get(sel_id) {
                        match fs_ops::move_to_trash(&item.path) {
                            Ok(_) => {
                                self.action_status = Some(format!("Trashed {}", item.name));
                                // Rescan after deletion
                                if let Some(root) = self.stats.root_path.clone() {
                                    self.start_scan(root);
                                }
                            }
                            Err(e) => self.action_status = Some(e),
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for LinDirStatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll scanner updates
        self.check_scanner_messages();

        // Keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            if let Some(root) = self.stats.root_path.clone() {
                self.start_scan(root);
            }
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.open_dialog.open(self.stats.root_path.clone());
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            self.handle_toolbar_action(ToolbarAction::TrashSelected);
        }

        // Top Menu Bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open... (Ctrl+O)").clicked() {
                        self.open_dialog.open(self.stats.root_path.clone());
                        ui.close_menu();
                    }
                    if ui.button("Refresh (F5)").clicked() {
                        if let Some(root) = self.stats.root_path.clone() {
                            self.start_scan(root);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let has_sel = self.selected_id.is_some();
                    ui.add_enabled_ui(has_sel, |ui| {
                        if ui.button("Copy Path").clicked() {
                            if let Some(sel_id) = self.selected_id {
                                if let Some(item) = self.model.get(sel_id) {
                                    ui.ctx().copy_text(item.path.display().to_string());
                                    self.action_status = Some("Path copied".to_string());
                                }
                            }
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Clean Up", |ui| {
                    let has_sel = self.selected_id.is_some();
                    ui.add_enabled_ui(has_sel, |ui| {
                        if ui.button("Open Item").clicked() {
                            if let Some(sel_id) = self.selected_id {
                                if let Some(item) = self.model.get(sel_id) {
                                    let _ = fs_ops::open_file(&item.path);
                                }
                            }
                            ui.close_menu();
                        }
                        if ui.button("Reveal in File Manager").clicked() {
                            self.handle_toolbar_action(ToolbarAction::RevealInFileManager);
                            ui.close_menu();
                        }
                        if ui.button("Open Terminal Here").clicked() {
                            self.handle_toolbar_action(ToolbarAction::OpenTerminal);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Move to Trash (Del)").clicked() {
                            self.handle_toolbar_action(ToolbarAction::TrashSelected);
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Treemap", |ui| {
                    if ui.checkbox(&mut self.treemap_widget.options.cushion, "Cushion Shading (Per-Cell)").clicked() {
                        self.treemap_widget.invalidate();
                    }
                    if ui.checkbox(&mut self.treemap_widget.options.grid, "Show Grid").clicked() {
                        self.treemap_widget.invalidate();
                    }
                    ui.separator();
                    ui.label("Cushion Depth:");
                    if ui.radio_value(&mut self.treemap_widget.options.intensity, crate::treemap::CushionIntensity::Light, "Light").clicked() {
                        self.treemap_widget.invalidate();
                    }
                    if ui.radio_value(&mut self.treemap_widget.options.intensity, crate::treemap::CushionIntensity::Medium, "Medium (Classic)").clicked() {
                        self.treemap_widget.invalidate();
                    }
                    if ui.radio_value(&mut self.treemap_widget.options.intensity, crate::treemap::CushionIntensity::Strong, "Strong (Deep)").clicked() {
                        self.treemap_widget.invalidate();
                    }
                    ui.separator();
                    if ui.button("Zoom In (Make Root)").clicked() {
                        self.handle_toolbar_action(ToolbarAction::ZoomIn);
                        ui.close_menu();
                    }
                    if ui.button("Reset Zoom").clicked() {
                        self.handle_toolbar_action(ToolbarAction::ZoomOut);
                        ui.close_menu();
                    }
                });

                ui.menu_button("Options", |ui| {
                    ui.checkbox(&mut self.options.same_fs, "Don't cross filesystem boundaries");
                    ui.checkbox(&mut self.options.exclude_kernel_fs, "Exclude Linux virtual filesystems (/proc, /sys)");
                    ui.checkbox(&mut self.options.follow_symlinks, "Follow symlinks");
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About LinDirStat").clicked() {
                        self.show_about_dialog = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Toolbar Panel
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            let is_scanning = self.stats.status == ScanStatus::Scanning;
            let has_sel = self.selected_id.is_some();
            let can_zoom_out = self.treemap_widget.zoom_root_id.is_some();

            if let Some(action) = render_toolbar(
                ui,
                is_scanning,
                self.treemap_widget.options.cushion,
                self.treemap_widget.options.grid,
                has_sel,
                can_zoom_out,
            ) {
                self.handle_toolbar_action(action);
            }
        });

        // Status Bar Panel (Bottom)
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            let selected_item = self.selected_id.and_then(|id| self.model.get(id));
            render_status_bar(
                ui,
                &mut self.stats,
                selected_item,
                &mut self.pacman,
                self.action_status.as_deref(),
            );
        });

        // Expand parents and request scroll whenever selected_id changes
        if self.selected_id != self.last_selected_id {
            self.last_selected_id = self.selected_id;
            if let Some(sel_id) = self.selected_id {
                let mut cur = self.model.get(sel_id).and_then(|i| i.parent);
                while let Some(pid) = cur {
                    if let Some(p) = self.model.get_mut(pid) {
                        p.expanded = true;
                        cur = p.parent;
                    } else {
                        break;
                    }
                }
                self.dir_tree_view.request_scroll_to_selected = true;
            }
        }

        // Main Central Panel (Upper Directory Tree + Extension List; Lower Treemap Visualizer)
        egui::CentralPanel::default().show(ctx, |ui| {
            let total_height = ui.available_height();
            let top_height = (total_height * self.top_bottom_split).clamp(100.0, total_height - 100.0);
            let bottom_height = total_height - top_height - 6.0;

            // 1. Upper Section: Directory Tree (Left) and Extension List (Right)
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), top_height),
                egui::Layout::left_to_right(egui::Align::Min),
                |ui| {
                    let total_width = ui.available_width();
                    let left_width = (total_width * self.left_right_split).clamp(200.0, total_width - 150.0);
                    let right_width = total_width - left_width - 8.0;

                    // Left Pane: Directory Tree View (MUST be top_down layout with unique ID!)
                    ui.push_id("dir_tree_panel", |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(left_width, top_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                let is_scanning = self.stats.status == ScanStatus::Scanning;
                                self.dir_tree_view.ui(
                                    ui,
                                    &mut self.model,
                                    &mut self.selected_id,
                                    &mut self.action_status,
                                    is_scanning,
                                );
                            },
                        );
                    });

                    ui.separator();

                    // Right Pane: Extension List View (MUST be top_down layout with unique ID!)
                    ui.push_id("ext_list_panel", |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(right_width, top_height),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                self.ext_list_view.ui(
                                    ui,
                                    &self.model,
                                    &mut self.selected_id,
                                );
                            },
                        );
                    });
                },
            );

            // Resizable separator handle between top and bottom
            ui.separator();

            // 2. Lower Section: Squarified Cushion Treemap Visualizer
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), bottom_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    self.treemap_widget.ui(
                        ui,
                        &self.model,
                        &mut self.selected_id,
                        &mut self.action_status,
                    );
                },
            );
        });

        // Folder / Drive Picker Dialog
        if let Some(chosen_dir) = self.open_dialog.show(ctx) {
            self.start_scan(chosen_dir);
        }

        // About Dialog
        if self.show_about_dialog {
            egui::Window::new("About LinDirStat")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("LinDirStat");
                    ui.label("A near 1-1 WinDirStat clone for Linux, built with Rust and egui.");
                    ui.add_space(8.0);
                    ui.label("Features:");
                    ui.label("• High-performance multi-threaded Linux scanner (ncdu-inspired)");
                    ui.label("• Squarified 3D cushion treemap visualization (van Wijk)");
                    ui.label("• Extension legend with WinDirStat palette ranking");
                    ui.label("• Classic chomping Pacman animation during scan");
                    ui.label("• Native Linux desktop integration (Trash, terminal, file manager)");
                    ui.add_space(12.0);
                    if ui.button("Close").clicked() {
                        self.show_about_dialog = false;
                    }
                });
        }
    }
}
