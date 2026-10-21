// Per-cell 3D Cushion shading algorithm matching WinDirStat

use egui::{Color32, ColorImage};
use super::layout::{CushionConfig, TreemapItem};
use crate::model::extension::ExtensionRegistry;
use crate::model::item::Item;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CushionIntensity {
    Light,
    Medium,
    Strong,
}

pub struct RenderOptions {
    pub cushion: bool,
    pub grid: bool,
    pub intensity: CushionIntensity,
    pub config: CushionConfig,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            cushion: true,
            grid: true,
            intensity: CushionIntensity::Medium,
            config: CushionConfig::default(),
        }
    }
}

/// Renders the treemap items into an egui::ColorImage with per-cell 3D cushion shading
pub fn render_cushion_treemap(
    items: &[Item],
    treemap_items: &[TreemapItem],
    extensions: &ExtensionRegistry,
    width: usize,
    height: usize,
    options: &RenderOptions,
) -> ColorImage {
    if width == 0 || height == 0 {
        return ColorImage::new([1, 1], Color32::BLACK);
    }

    // Canvas background
    let mut pixels = vec![Color32::from_rgb(18, 18, 22); width * height];

    // Light source coming from top-left (classic WinDirStat lighting)
    let light = options.config.light_dir;
    let light_len = (light[0] * light[0] + light[1] * light[1] + light[2] * light[2]).sqrt();
    let (lx, ly, lz) = if light_len > 0.0 {
        (light[0] / light_len, light[1] / light_len, light[2] / light_len)
    } else {
        (-0.437, -0.437, 0.786)
    };

    let ia = options.config.ambient_light;
    let is = 1.0 - ia;

    let ridge_mult = match options.intensity {
        CushionIntensity::Light => 1.8f32,
        CushionIntensity::Medium => 3.2f32,
        CushionIntensity::Strong => 4.8f32,
    };
    let ridge_scale = (options.config.height as f32 * ridge_mult).max(0.1);

    for tm_item in treemap_items {
        let item = match items.get(tm_item.item_id) {
            Some(i) => i,
            None => continue,
        };

        let color = if item.is_dir {
            Color32::from_rgb(60, 60, 65)
        } else {
            extensions.get_color_for_ext(item.extension.as_deref())
        };

        let r_base = color.r() as f32;
        let g_base = color.g() as f32;
        let b_base = color.b() as f32;

        let rx_min = (tm_item.rect.min.x.floor() as usize).clamp(0, width);
        let rx_max = (tm_item.rect.max.x.ceil() as usize).clamp(0, width);
        let ry_min = (tm_item.rect.min.y.floor() as usize).clamp(0, height);
        let ry_max = (tm_item.rect.max.y.ceil() as usize).clamp(0, height);

        if rx_min >= rx_max || ry_min >= ry_max {
            continue;
        }

        let cell_w = (rx_max - rx_min) as f32;
        let cell_h = (ry_max - ry_min) as f32;
        let inv_w = 1.0 / cell_w;
        let inv_h = 1.0 / cell_h;

        // Render each file cell with its own individual cushion dome
        for y in ry_min..ry_max {
            let row_offset = y * width;
            let is_y_edge = options.grid && cell_h >= 3.0 && (y == ry_min || y == ry_max - 1);

            let v = (y as f32 - ry_min as f32 + 0.5) * inv_h;
            let dy = (v - 0.5) * 2.0; // [-1.0, 1.0] from top to bottom
            let ny = dy * ridge_scale;

            for x in rx_min..rx_max {
                let is_x_edge = options.grid && cell_w >= 3.0 && (x == rx_min || x == rx_max - 1);

                if is_x_edge || is_y_edge {
                    // Crisp 1px dark grid border separating file pillows
                    pixels[row_offset + x] = Color32::from_rgb(20, 20, 24);
                    continue;
                }

                if !options.cushion {
                    pixels[row_offset + x] = color;
                    continue;
                }

                let u = (x as f32 - rx_min as f32 + 0.5) * inv_w;
                let dx = (u - 0.5) * 2.0; // [-1.0, 1.0] from left to right
                let nx = dx * ridge_scale;

                // Per-cell surface normal pointing towards top-left on top-left of cell,
                // and pointing towards bottom-right on bottom-right of cell.
                let dot = nx * lx + ny * ly + lz;
                let denom = (nx * nx + ny * ny + 1.0).sqrt();

                let mut cosa = if denom > 0.0 { dot / denom } else { 0.0 };
                cosa = cosa.clamp(-1.0, 1.0);

                let mut intensity = ia + is * cosa.max(0.0);
                intensity = intensity.clamp(0.0, 1.0);

                let r = (r_base * intensity).clamp(0.0, 255.0) as u8;
                let g = (g_base * intensity).clamp(0.0, 255.0) as u8;
                let b = (b_base * intensity).clamp(0.0, 255.0) as u8;

                pixels[row_offset + x] = Color32::from_rgb(r, g, b);
            }
        }
    }

    ColorImage {
        size: [width, height],
        pixels,
    }
}
