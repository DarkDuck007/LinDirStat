// Classic WinDirStat Pacman animation widget

use egui::{Color32, Pos2, Stroke, Ui, Vec2};

pub struct PacmanWidget {
    pub active: bool,
    pub progress: f32, // 0.0 to 1.0 (or continuous)
}

impl PacmanWidget {
    pub fn new(active: bool) -> Self {
        Self {
            active,
            progress: 0.0,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, width: f32, height: f32) {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::hover());
        let painter = ui.painter();

        // Draw track background
        painter.rect_filled(rect, 3.0, Color32::from_rgb(30, 30, 36));
        painter.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, Color32::from_rgb(60, 60, 70)), egui::StrokeKind::Inside);

        if !self.active {
            return;
        }

        let time = ui.input(|i| i.time) as f32;
        let speed = 60.0; // pixels per second
        let usable_width = (rect.width() - 24.0).max(10.0);
        let cycle_x = (time * speed) % (usable_width * 2.0);
        let (pacman_x, moving_right) = if cycle_x <= usable_width {
            (rect.min.x + 12.0 + cycle_x, true)
        } else {
            (rect.min.x + 12.0 + (2.0 * usable_width - cycle_x), false)
        };

        let pacman_center = Pos2::new(pacman_x, rect.center().y);
        let radius = (height * 0.38).clamp(5.0, 10.0);

        // Draw pellets (dots)
        let dot_spacing = 18.0;
        let mut dot_x = rect.min.x + 12.0;
        while dot_x <= rect.max.x - 12.0 {
            // Draw dot only if not right under pacman
            if (dot_x - pacman_x).abs() > radius * 0.7 {
                painter.circle_filled(
                    Pos2::new(dot_x, rect.center().y),
                    2.2,
                    Color32::from_rgb(255, 220, 100),
                );
            }
            dot_x += dot_spacing;
        }

        // Animated mouth angle (oscillates between 10 and 65 degrees)
        let mouth_open = ((time * 8.0).sin().abs() * 0.7 + 0.15) * std::f32::consts::PI / 2.0;
        let base_angle = if moving_right { 0.0 } else { std::f32::consts::PI };

        let start_angle = base_angle + mouth_open / 2.0;
        let end_angle = base_angle + 2.0 * std::f32::consts::PI - mouth_open / 2.0;

        // Draw Pacman as a yellow mesh sector
        let num_points = 24;
        let mut points = vec![pacman_center];
        for i in 0..=num_points {
            let frac = i as f32 / num_points as f32;
            let a = start_angle + frac * (end_angle - start_angle);
            points.push(Pos2::new(
                pacman_center.x + a.cos() * radius,
                pacman_center.y + a.sin() * radius,
            ));
        }

        let yellow = Color32::from_rgb(255, 230, 20);
        let mut mesh = egui::Mesh::default();
        for p in &points {
            mesh.colored_vertex(*p, yellow);
        }
        for i in 1..num_points + 1 {
            mesh.add_triangle(0, i as u32, (i + 1) as u32);
        }

        painter.add(mesh);

        // Pacman eye
        let eye_offset_x = if moving_right { -radius * 0.15 } else { radius * 0.15 };
        let eye_pos = Pos2::new(pacman_center.x + eye_offset_x, pacman_center.y - radius * 0.45);
        painter.circle_filled(eye_pos, 1.5, Color32::BLACK);

        // Keep UI redrawing while scanning
        ui.ctx().request_repaint();
    }
}
