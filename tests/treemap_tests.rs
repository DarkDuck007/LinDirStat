use egui::{pos2, Rect};
use lindirstat::model::extension::ExtensionRegistry;
use lindirstat::model::item::Item;
use lindirstat::treemap::cushion::{render_cushion_treemap, RenderOptions};
use lindirstat::treemap::layout::{add_ridge, layout_treemap, squarify, CushionConfig};
use std::path::PathBuf;

#[test]
fn test_squarify_partitioning_and_aspect_ratios() {
    let bounds = Rect::from_min_max(pos2(0.0, 0.0), pos2(1000.0, 800.0));
    let weights = vec![500.0, 300.0, 200.0, 100.0, 50.0, 25.0, 10.0, 5.0];

    let rects = squarify(bounds, &weights);
    assert_eq!(rects.len(), weights.len());

    let mut total_child_area = 0.0f32;
    for r in &rects {
        assert!(r.width() > 0.0, "Width must be positive");
        assert!(r.height() > 0.0, "Height must be positive");
        assert!(r.min.x >= bounds.min.x - 0.5);
        assert!(r.max.x <= bounds.max.x + 0.5);
        assert!(r.min.y >= bounds.min.y - 0.5);
        assert!(r.max.y <= bounds.max.y + 0.5);
        total_child_area += r.width() * r.height();
    }

    let parent_area = bounds.width() * bounds.height();
    let diff = (total_child_area - parent_area).abs();
    assert!(
        diff / parent_area < 0.05,
        "Total rects area {} should match parent area {}",
        total_child_area,
        parent_area
    );
}

#[test]
fn test_cushion_ridge_math() {
    let mut surface = [0.0f64; 4];
    let rect = Rect::from_min_max(pos2(10.0, 20.0), pos2(110.0, 120.0));
    let h = 0.50;

    add_ridge(rect, &mut surface, h);

    // After adding parabolic ridge:
    // width = 100, height = 100
    // h4 = 2.0
    // surface[0] should be -wf = -2.0 / 100 = -0.02
    // surface[1] should be -hf = -2.0 / 100 = -0.02
    // surface[2] should be wf * (110 + 10) = 0.02 * 120 = 2.4
    // surface[3] should be hf * (120 + 20) = 0.02 * 140 = 2.8
    assert!((surface[0] - (-0.02)).abs() < 1e-6);
    assert!((surface[1] - (-0.02)).abs() < 1e-6);
    assert!((surface[2] - 2.4).abs() < 1e-6);
    assert!((surface[3] - 2.8).abs() < 1e-6);
}

#[test]
fn test_treemap_render_buffer_generation() {
    let mut items = Vec::new();
    let root = Item::new_dir(0, PathBuf::from("/test"), 0, 1, 1, None);
    let f1 = Item::new_file(1, PathBuf::from("/test/video.mp4"), 8000, 8192, 0, 1, 2, Some(0));
    let f2 = Item::new_file(2, PathBuf::from("/test/audio.mp3"), 2000, 2048, 0, 1, 3, Some(0));

    items.push(root);
    items.push(f1);
    items.push(f2);
    items[0].children = vec![1, 2];
    items[0].size = 10000;

    let extensions = ExtensionRegistry::build_from_items(&items, 10000);
    let bounds = Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 150.0));
    let tm_items = layout_treemap(&items, 0, bounds, CushionConfig::default());

    assert!(!tm_items.is_empty());

    let img = render_cushion_treemap(
        &items,
        &tm_items,
        &extensions,
        200,
        150,
        &RenderOptions::default(),
    );

    assert_eq!(img.size, [200, 150]);
    assert_eq!(img.pixels.len(), 200 * 150);

    // Verify non-zero pixels rendered
    let has_colored_pixels = img.pixels.iter().any(|p| p.r() > 30 || p.g() > 30 || p.b() > 30);
    assert!(has_colored_pixels);
}

#[test]
fn test_per_cell_cushion_gradient_uniqueness() {
    let mut items = Vec::new();
    let root = Item::new_dir(0, PathBuf::from("/test"), 0, 1, 1, None);
    // Two files of identical size and extension side-by-side
    let f1 = Item::new_file(1, PathBuf::from("/test/file1.mp4"), 5000, 5120, 0, 1, 2, Some(0));
    let f2 = Item::new_file(2, PathBuf::from("/test/file2.mp4"), 5000, 5120, 0, 1, 3, Some(0));

    items.push(root);
    items.push(f1);
    items.push(f2);
    items[0].children = vec![1, 2];
    items[0].size = 10000;

    let extensions = ExtensionRegistry::build_from_items(&items, 10000);
    let bounds = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 50.0));
    let tm_items = layout_treemap(&items, 0, bounds, CushionConfig::default());

    let img = render_cushion_treemap(
        &items,
        &tm_items,
        &extensions,
        100,
        50,
        &RenderOptions {
            cushion: true,
            grid: false,
            intensity: lindirstat::treemap::CushionIntensity::Medium,
            config: CushionConfig::default(),
        },
    );

    // tm_items has 2 leaves: one on left [0..50], one on right [50..100]
    // Check pixel at top-left of cell 1 (bright) vs bottom-right of cell 1 (shaded)
    let p_cell1_top_left = img.pixels[5 * 100 + 5];
    let p_cell1_bot_right = img.pixels[45 * 100 + 45];
    assert!(
        p_cell1_top_left.b() > p_cell1_bot_right.b(),
        "Cell 1 top-left ({}) must be brighter than its bottom-right ({})",
        p_cell1_top_left.b(),
        p_cell1_bot_right.b()
    );

    // Check pixel at top-left of cell 2 (bright) vs bottom-right of cell 2 (shaded)
    let p_cell2_top_left = img.pixels[5 * 100 + 55];
    let p_cell2_bot_right = img.pixels[45 * 100 + 95];
    assert!(
        p_cell2_top_left.b() > p_cell2_bot_right.b(),
        "Cell 2 top-left ({}) must be brighter than its bottom-right ({})",
        p_cell2_top_left.b(),
        p_cell2_bot_right.b()
    );

    // Boundary contrast: right edge of cell 1 vs left edge of cell 2
    // Cell 1 right edge is shaded; Cell 2 left edge has bright highlight!
    let p_boundary_left = img.pixels[25 * 100 + 47]; // cell 1 near right
    let p_boundary_right = img.pixels[25 * 100 + 53]; // cell 2 near left
    assert!(
        p_boundary_right.b() > p_boundary_left.b(),
        "Cell 2 left edge ({}) must be brighter than Cell 1 right edge ({}) due to per-cell cushion gradient",
        p_boundary_right.b(),
        p_boundary_left.b()
    );
}


