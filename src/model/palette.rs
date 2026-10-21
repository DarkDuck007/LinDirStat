// WinDirStat palette and color assignment for file extensions.
// Matches WinDirStat's classic vibrant extension colors.

use egui::Color32;

/// Classic WinDirStat primary extension palette
const BASE_PALETTE: &[[u8; 3]] = &[
    [0, 0, 255],     // Blue
    [255, 0, 0],     // Red
    [0, 255, 0],     // Green
    [255, 0, 255],   // Magenta
    [0, 255, 255],   // Cyan
    [255, 255, 0],   // Yellow
    [255, 128, 0],   // Orange
    [128, 0, 255],   // Purple
    [0, 160, 255],   // Sky Blue
    [0, 200, 100],   // Sea Green
    [255, 100, 100], // Coral
    [200, 200, 0],   // Olive
    [150, 0, 200],   // Violet
    [0, 200, 200],   // Dark Cyan
    [255, 170, 0],   // Amber
    [120, 220, 0],   // Chartreuse
    [220, 80, 140],  // Pink
    [80, 120, 240],  // Periwinkle
    [100, 180, 20],  // Grass Green
    [240, 140, 60],  // Tangerine
];

/// Computes an extension color from an index or generates one via golden-ratio HSL
pub fn get_palette_color(index: usize) -> Color32 {
    if index < BASE_PALETTE.len() {
        let [r, g, b] = BASE_PALETTE[index];
        Color32::from_rgb(r, g, b)
    } else {
        // Golden ratio hue shift for distinct remaining colors
        let golden_ratio = 0.618_033_988_749_895;
        let hue = ((index as f32 * golden_ratio) % 1.0) * 360.0;
        let [r, g, b] = hsl_to_rgb(hue, 0.85, 0.55);
        Color32::from_rgb(r, g, b)
    }
}

/// Convert HSL (h in [0, 360], s in [0, 1], l in [0, 1]) to RGB [0, 255]
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h / 60.0;
    let x = c * (1.0 - ((hp % 2.0) - 1.0).abs());
    let (r1, g1, b1) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = l - c / 2.0;
    [
        ((r1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).clamp(0.0, 255.0) as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_distinctness() {
        let c0 = get_palette_color(0);
        let c1 = get_palette_color(1);
        assert_ne!(c0, c1);
        let c_high = get_palette_color(100);
        assert!(c_high.r() > 0 || c_high.g() > 0 || c_high.b() > 0);
    }
}

