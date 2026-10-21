// Extension statistics and ranking for WinDirStat's top-right extension legend.

use egui::Color32;
use std::collections::HashMap;

use super::item::Item;
use super::palette::get_palette_color;

#[derive(Debug, Clone)]
pub struct ExtensionInfo {
    pub extension: String,
    pub color: Color32,
    pub total_bytes: u64,
    pub file_count: usize,
    pub percentage: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, Default)]
pub struct ExtensionRegistry {
    pub extensions: Vec<ExtensionInfo>,
    ext_to_index: HashMap<String, usize>,
}

impl ExtensionRegistry {
    pub fn build_from_items(items: &[Item], total_size: u64) -> Self {
        let mut map: HashMap<String, (u64, usize)> = HashMap::new();

        for item in items {
            if !item.is_dir && item.size > 0 {
                let ext = item.extension.clone().unwrap_or_else(|| "<Files>".to_string());
                let entry = map.entry(ext).or_insert((0, 0));
                entry.0 += item.size;
                entry.1 += 1;
            }
        }

        let mut list: Vec<(String, u64, usize)> = map.into_iter().map(|(e, (b, c))| (e, b, c)).collect();

        // Sort descending by total size
        list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let mut extensions = Vec::with_capacity(list.len());
        let mut ext_to_index = HashMap::with_capacity(list.len());

        let total_f = if total_size > 0 { total_size as f32 } else { 1.0 };

        for (idx, (ext, bytes, count)) in list.into_iter().enumerate() {
            let color = get_palette_color(idx);
            let percentage = (bytes as f32 / total_f).clamp(0.0, 1.0);
            ext_to_index.insert(ext.clone(), idx);
            extensions.push(ExtensionInfo {
                extension: ext,
                color,
                total_bytes: bytes,
                file_count: count,
                percentage,
            });
        }

        Self {
            extensions,
            ext_to_index,
        }
    }

    pub fn get_color_for_ext(&self, ext_opt: Option<&str>) -> Color32 {
        let key = ext_opt.unwrap_or("<Files>");
        if let Some(&idx) = self.ext_to_index.get(key) {
            self.extensions[idx].color
        } else {
            Color32::from_rgb(160, 160, 160)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extension_registry_ranking() {
        let items = vec![
            Item::new_file(0, PathBuf::from("a.mp4"), 1000, 1024, 0, 1, 1, None),
            Item::new_file(1, PathBuf::from("b.mp4"), 2000, 2048, 0, 1, 2, None),
            Item::new_file(2, PathBuf::from("c.txt"), 500, 1024, 0, 1, 3, None),
        ];

        let reg = ExtensionRegistry::build_from_items(&items, 3500);
        assert_eq!(reg.extensions.len(), 2);
        assert_eq!(reg.extensions[0].extension, ".mp4");
        assert_eq!(reg.extensions[0].total_bytes, 3000);
        assert_eq!(reg.extensions[0].file_count, 2);
        assert_eq!(reg.extensions[1].extension, ".txt");
        assert_eq!(reg.extensions[1].total_bytes, 500);
    }
}
