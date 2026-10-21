// WinDirStat-style "Select Drives or Folder" dialog for Linux

use egui::{Color32, RichText, Vec2, Window};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MountDrive {
    pub device: String,
    pub mount_point: PathBuf,
    pub fs_type: String,
}

pub struct OpenDialog {
    pub is_open: bool,
    pub path_input: String,
    pub current_browse_dir: PathBuf,
    pub drives: Vec<MountDrive>,
    pub selected_drive: Option<PathBuf>,
}

impl Default for OpenDialog {
    fn default() -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"));
        let drives = detect_mounts();

        Self {
            is_open: false,
            path_input: home.display().to_string(),
            current_browse_dir: home,
            drives,
            selected_drive: None,
        }
    }
}

impl OpenDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, default_path: Option<PathBuf>) {
        if let Some(p) = default_path {
            self.path_input = p.display().to_string();
            self.current_browse_dir = if p.is_dir() {
                p
            } else {
                p.parent().unwrap_or(&p).to_path_buf()
            };
        }
        self.drives = detect_mounts();
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Renders the dialog and returns Some(PathBuf) if the user confirms a directory to scan
    pub fn show(&mut self, ctx: &egui::Context) -> Option<PathBuf> {
        if !self.is_open {
            return None;
        }

        let mut confirmed_path = None;
        let mut close_dialog = false;

        Window::new("Select Drives or Folder to Scan")
            .collapsible(false)
            .resizable(true)
            .default_size(Vec2::new(650.0, 480.0))
            .min_size(Vec2::new(500.0, 380.0))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);

                ui.heading("Select where to scan:");
                ui.separator();

                // 1. Mounted Drives / Devices section (like WinDirStat Drive list)
                if !self.drives.is_empty() {
                    ui.label(RichText::new("Mounted Drives / Filesystems:").strong());
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        for drive in &self.drives {
                            let is_sel = self.path_input == drive.mount_point.display().to_string();
                            let label = format!(
                                "💾 {} [{}] ({})",
                                drive.mount_point.display(),
                                drive.device,
                                drive.fs_type
                            );
                            if ui.selectable_label(is_sel, label).clicked() {
                                self.path_input = drive.mount_point.display().to_string();
                                self.current_browse_dir = drive.mount_point.clone();
                            }
                        }
                    });
                    ui.add_space(4.0);
                }

                // 2. Path text input and quick shortcuts
                ui.label(RichText::new("Folder Path:").strong());
                ui.horizontal(|ui| {
                    let text_edit = egui::TextEdit::singleline(&mut self.path_input)
                        .desired_width(ui.available_width() - 80.0);
                    if ui.add(text_edit).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let p = PathBuf::from(&self.path_input);
                        if p.exists() {
                            confirmed_path = Some(p);
                            close_dialog = true;
                        }
                    }

                    // Native system picker button as fallback/option
                    if ui.button("System...").on_hover_text("Try system native file chooser (if supported by desktop environment)").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.path_input = folder.display().to_string();
                            self.current_browse_dir = folder;
                        }
                    }
                });

                // Quick buttons: Home, Root, Current, Temp
                ui.horizontal(|ui| {
                    ui.label("Shortcuts:");
                    if let Ok(home) = std::env::var("HOME") {
                        if ui.button("🏠 Home").clicked() {
                            self.path_input = home.clone();
                            self.current_browse_dir = PathBuf::from(home);
                        }
                    }
                    if ui.button("💻 Root (/)").clicked() {
                        self.path_input = "/".to_string();
                        self.current_browse_dir = PathBuf::from("/");
                    }
                    if let Ok(cur) = std::env::current_dir() {
                        if ui.button("📂 Current").clicked() {
                            self.path_input = cur.display().to_string();
                            self.current_browse_dir = cur;
                        }
                    }
                    if ui.button("📁 /tmp").clicked() {
                        self.path_input = "/tmp".to_string();
                        self.current_browse_dir = PathBuf::from("/tmp");
                    }
                });

                ui.separator();

                // 3. Interactive Directory Browser
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Browser:").strong());
                    ui.monospace(self.current_browse_dir.display().to_string());
                    if self.current_browse_dir.parent().is_some() {
                        if ui.button("⬆ Parent Directory").clicked() {
                            if let Some(parent) = self.current_browse_dir.parent() {
                                self.current_browse_dir = parent.to_path_buf();
                                self.path_input = self.current_browse_dir.display().to_string();
                            }
                        }
                    }
                });

                let browser_height = ui.available_height() - 45.0;
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(browser_height)
                        .show(ui, |ui| {
                            let mut subdirs = Vec::new();
                            if let Ok(entries) = std::fs::read_dir(&self.current_browse_dir) {
                                for entry in entries.flatten() {
                                    if let Ok(ft) = entry.file_type() {
                                        if ft.is_dir() {
                                            subdirs.push(entry.file_name().to_string_lossy().to_string());
                                        }
                                    }
                                }
                            }
                            subdirs.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

                            if subdirs.is_empty() {
                                ui.label(RichText::new("No subdirectories found or permission denied").color(Color32::from_rgb(140, 140, 140)));
                            } else {
                                for name in subdirs {
                                    let btn_label = format!("📁 {}", name);
                                    if ui.button(btn_label).clicked() {
                                        let next_dir = self.current_browse_dir.join(&name);
                                        self.current_browse_dir = next_dir.clone();
                                        self.path_input = next_dir.display().to_string();
                                    }
                                }
                            }
                        });
                });

                ui.separator();

                // 4. Action buttons: Scan and Cancel
                ui.horizontal(|ui| {
                    let target_path = PathBuf::from(&self.path_input);
                    let valid_dir = target_path.exists() && target_path.is_dir();

                    ui.add_enabled_ui(valid_dir, |ui| {
                        if ui.button(RichText::new("🚀 Start Scan").strong()).clicked() {
                            confirmed_path = Some(target_path);
                            close_dialog = true;
                        }
                    });

                    if !valid_dir {
                        ui.label(RichText::new("Path is not a valid directory").color(Color32::from_rgb(220, 100, 100)));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            close_dialog = true;
                        }
                    });
                });
            });

        if close_dialog {
            self.is_open = false;
        }

        confirmed_path
    }
}

/// Discovers mounted Linux filesystems from /proc/mounts
fn detect_mounts() -> Vec<MountDrive> {
    let mut mounts = Vec::new();
    let Ok(content) = std::fs::read_to_string("/proc/mounts") else {
        return vec![MountDrive {
            device: "/dev/root".to_string(),
            mount_point: PathBuf::from("/"),
            fs_type: "ext4".to_string(),
        }];
    };

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let device = parts[0];
            let mount_point = parts[1];
            let fs_type = parts[2];

            // Filter for physical / block / media mounts
            if device.starts_with("/dev/") && !mount_point.starts_with("/dev") && !mount_point.starts_with("/proc") && !mount_point.starts_with("/sys") {
                mounts.push(MountDrive {
                    device: device.to_string(),
                    mount_point: PathBuf::from(mount_point),
                    fs_type: fs_type.to_string(),
                });
            }
        }
    }

    if mounts.is_empty() {
        mounts.push(MountDrive {
            device: "root".to_string(),
            mount_point: PathBuf::from("/"),
            fs_type: "rootfs".to_string(),
        });
    }

    mounts
}
