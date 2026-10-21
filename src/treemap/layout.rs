// Squarified Treemap layout algorithm (Bruls, Huizing, van Wijk) matching WinDirStat

use egui::Rect;

#[derive(Debug, Clone)]
pub struct TreemapItem {
    pub item_id: usize,
    pub rect: Rect,
    pub is_dir: bool,
    pub surface: [f64; 4],
    pub depth: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct CushionConfig {
    pub ambient_light: f32, // e.g. 0.15
    pub height: f64,        // e.g. 0.40
    pub scale_factor: f64,  // e.g. 0.85
    pub light_dir: [f32; 3],// e.g. [-1.0, -1.0, 2.0]
}

impl Default for CushionConfig {
    fn default() -> Self {
        Self {
            ambient_light: 0.20,
            height: 0.50,
            scale_factor: 0.85,
            light_dir: [-1.0, -1.0, 2.0],
        }
    }
}

/// Recursively computes squarified treemap layout for a hierarchy
pub fn layout_treemap(
    items: &[crate::model::item::Item],
    root_id: usize,
    bounds: Rect,
    config: CushionConfig,
) -> Vec<TreemapItem> {
    if bounds.width() < 2.0 || bounds.height() < 2.0 {
        return Vec::new();
    }

    let mut result = Vec::new();
    let initial_surface = [0.0f64; 4];

    let mut stack: Vec<(usize, Rect, [f64; 4], f64, usize)> = vec![(
        root_id,
        bounds,
        initial_surface,
        config.height,
        0,
    )];

    while let Some((node_id, rect, mut surface, ridge_h, depth)) = stack.pop() {
        if rect.width() < 1.0 || rect.height() < 1.0 {
            continue;
        }

        // Add parabolic ridge for this level
        add_ridge(rect, &mut surface, ridge_h);

        let node = match items.get(node_id) {
            Some(n) => n,
            None => continue,
        };

        if !node.is_dir {
            result.push(TreemapItem {
                item_id: node_id,
                rect,
                is_dir: false,
                surface,
                depth,
            });
            continue;
        }

        // Filter valid children with size > 0
        let valid_children: Vec<usize> = node
            .children
            .iter()
            .copied()
            .filter(|&cid| items.get(cid).map(|c| c.size > 0).unwrap_or(false))
            .collect();

        if valid_children.is_empty() {
            // Empty dir or dir with 0-size files: register as leaf
            result.push(TreemapItem {
                item_id: node_id,
                rect,
                is_dir: true,
                surface,
                depth,
            });
            continue;
        }

        let weights: Vec<f64> = valid_children
            .iter()
            .map(|&cid| items[cid].size as f64)
            .collect();

        let child_rects = squarify(rect, &weights);

        let next_ridge = ridge_h * config.scale_factor;

        for (cid, child_rect) in valid_children.into_iter().zip(child_rects) {
            if child_rect.width() >= 1.0 && child_rect.height() >= 1.0 {
                stack.push((cid, child_rect, surface, next_ridge, depth + 1));
            }
        }
    }

    result
}

/// Adds a parabolic ridge to the surface normal equation (van Wijk cushion treemap)
pub fn add_ridge(rc: Rect, surface: &mut [f64; 4], h: f64) {
    let w = rc.width() as f64;
    let height = rc.height() as f64;
    if w <= 0.0 || height <= 0.0 {
        return;
    }

    let h4 = 4.0 * h;

    let wf = h4 / w;
    surface[2] += wf * (rc.max.x as f64 + rc.min.x as f64);
    surface[0] -= wf;

    let hf = h4 / height;
    surface[3] += hf * (rc.max.y as f64 + rc.min.y as f64);
    surface[1] -= hf;
}

/// Squarifies a list of weights into a parent rectangle
pub fn squarify(bounds: Rect, weights: &[f64]) -> Vec<Rect> {
    if weights.is_empty() {
        return Vec::new();
    }

    let total_weight: f64 = weights.iter().sum();
    if total_weight <= 0.0 {
        return vec![Rect::NOTHING; weights.len()];
    }

    let total_area = bounds.width() as f64 * bounds.height() as f64;
    let normalized_weights: Vec<f64> = weights
        .iter()
        .map(|&w| (w / total_weight) * total_area)
        .collect();

    let mut rects = vec![Rect::NOTHING; weights.len()];
    squarify_recursive(bounds, &normalized_weights, 0, &mut rects);
    rects
}

fn squarify_recursive(
    remaining_rect: Rect,
    weights: &[f64],
    start_index: usize,
    results: &mut [Rect],
) {
    if weights.is_empty() || remaining_rect.width() < 1.0 || remaining_rect.height() < 1.0 {
        return;
    }

    let w = remaining_rect.width() as f64;
    let h = remaining_rect.height() as f64;
    let shorter_side = w.min(h);

    if shorter_side <= 0.0 {
        return;
    }

    let mut row_len = 1;
    let mut current_worst = worst_aspect_ratio(&weights[..row_len], shorter_side);

    while row_len < weights.len() {
        let next_worst = worst_aspect_ratio(&weights[..row_len + 1], shorter_side);
        if next_worst <= current_worst {
            current_worst = next_worst;
            row_len += 1;
        } else {
            break;
        }
    }

    let (cut_rect, leftover_rect) = layout_row(
        remaining_rect,
        &weights[..row_len],
        &mut results[start_index..start_index + row_len],
    );

    let _ = cut_rect;
    squarify_recursive(
        leftover_rect,
        &weights[row_len..],
        start_index + row_len,
        results,
    );
}

fn worst_aspect_ratio(row_weights: &[f64], shorter_side: f64) -> f64 {
    let sum: f64 = row_weights.iter().sum();
    if sum <= 0.0 || shorter_side <= 0.0 {
        return f64::MAX;
    }

    let side_sq = shorter_side * shorter_side;
    let sum_sq = sum * sum;

    let mut max_aspect: f64 = 0.0;
    for &w in row_weights {
        let a1 = (side_sq * w) / sum_sq;
        let a2 = sum_sq / (side_sq * w);
        let aspect = a1.max(a2);
        if aspect > max_aspect {
            max_aspect = aspect;
        }
    }

    max_aspect
}

fn layout_row(bounds: Rect, row_weights: &[f64], dest: &mut [Rect]) -> (Rect, Rect) {
    let row_sum: f64 = row_weights.iter().sum();
    let is_horizontal = bounds.width() >= bounds.height();

    if is_horizontal {
        let row_width = (row_sum / bounds.height() as f64).clamp(0.0, bounds.width() as f64) as f32;
        let mut cur_y = bounds.min.y;

        for (i, &w) in row_weights.iter().enumerate() {
            let h = if row_sum > 0.0 {
                ((w / row_sum) * bounds.height() as f64) as f32
            } else {
                0.0
            };
            let child_rect = Rect::from_min_max(
                egui::pos2(bounds.min.x, cur_y),
                egui::pos2(bounds.min.x + row_width, (cur_y + h).min(bounds.max.y)),
            );
            dest[i] = child_rect;
            cur_y += h;
        }

        let cut_rect = Rect::from_min_max(bounds.min, egui::pos2(bounds.min.x + row_width, bounds.max.y));
        let leftover = Rect::from_min_max(
            egui::pos2(bounds.min.x + row_width, bounds.min.y),
            bounds.max,
        );
        (cut_rect, leftover)
    } else {
        let row_height = (row_sum / bounds.width() as f64).clamp(0.0, bounds.height() as f64) as f32;
        let mut cur_x = bounds.min.x;

        for (i, &w) in row_weights.iter().enumerate() {
            let item_w = if row_sum > 0.0 {
                ((w / row_sum) * bounds.width() as f64) as f32
            } else {
                0.0
            };
            let child_rect = Rect::from_min_max(
                egui::pos2(cur_x, bounds.min.y),
                egui::pos2((cur_x + item_w).min(bounds.max.x), bounds.min.y + row_height),
            );
            dest[i] = child_rect;
            cur_x += item_w;
        }

        let cut_rect = Rect::from_min_max(bounds.min, egui::pos2(bounds.max.x, bounds.min.y + row_height));
        let leftover = Rect::from_min_max(
            egui::pos2(bounds.min.x, bounds.min.y + row_height),
            bounds.max,
        );
        (cut_rect, leftover)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squarify_weights() {
        let bounds = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(100.0, 100.0));
        let weights = vec![50.0, 25.0, 25.0];
        let rects = squarify(bounds, &weights);
        assert_eq!(rects.len(), 3);
        for r in &rects {
            assert!(r.width() > 0.0);
            assert!(r.height() > 0.0);
            assert!(r.min.x >= 0.0 && r.max.x <= 100.1);
            assert!(r.min.y >= 0.0 && r.max.y <= 100.1);
        }
    }
}

