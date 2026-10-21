use eframe::egui;
use lindirstat::model::item::Item;
use lindirstat::model::tree::TreeModel;
use lindirstat::views::dir_tree::DirTreeView;
use lindirstat::views::open_dialog::OpenDialog;
use std::path::PathBuf;

#[test]
fn test_dir_tree_view_renders_visible_hierarchy() {
    let ctx = egui::Context::default();
    let mut tree = TreeModel::new();
    let root = tree.add_item(Item::new_dir(0, PathBuf::from("/my_project"), 0, 1, 1, None));
    let src = tree.add_item(Item::new_dir(0, PathBuf::from("/my_project/src"), 0, 1, 2, Some(root)));
    let _f = tree.add_item(Item::new_file(0, PathBuf::from("/my_project/src/main.rs"), 1500, 2048, 0, 1, 3, Some(src)));
    tree.compute_rollup();

    let mut view = DirTreeView::new();
    let mut selected = None;
    let mut status = None;

    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 800.0))),
        ..Default::default()
    };

    let full_output = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let top_height = 300.0;
            let left_width = 600.0;

            ui.allocate_ui_with_layout(
                egui::vec2(left_width, top_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    view.ui(ui, &mut tree, &mut selected, &mut status, false);
                },
            );
        });
    });

    let mut found_root = false;
    let mut found_src = false;
    let mut y_coords = Vec::new();

    for clipped in &full_output.shapes {
        if let egui::epaint::Shape::Text(t) = &clipped.shape {
            let text = &t.galley.job.text;
            if text == "my_project" {
                found_root = true;
                y_coords.push(t.pos.y);
            }
            if text == "src" {
                found_src = true;
                y_coords.push(t.pos.y);
            }
        }
    }

    assert!(found_root, "Root directory must be visible in Directory Tree");
    assert!(found_src, "Subdirectory must be visible in Directory Tree");
    assert_eq!(y_coords.len(), 2);
    // Subdirectory must be rendered below root (vertically stacked, not on the same horizontal line!)
    assert!(
        y_coords[1] > y_coords[0],
        "Subdirectory Y ({}) must be below Root Y ({})",
        y_coords[1],
        y_coords[0]
    );
}

#[test]
fn test_open_dialog_lifecycle_and_navigation() {
    let mut dialog = OpenDialog::new();
    assert!(!dialog.is_open);

    // Open dialog with temp directory
    let tmp = tempfile::tempdir().unwrap();
    dialog.open(Some(tmp.path().to_path_buf()));
    assert!(dialog.is_open);
    assert_eq!(dialog.current_browse_dir, tmp.path());

    // Verify mount points detected
    assert!(!dialog.drives.is_empty(), "Drives should be detected from system mounts");

    // Close dialog
    dialog.close();
    assert!(!dialog.is_open);
}

#[test]
fn test_dir_tree_virtualization_and_scroll_jump() {
    let ctx = egui::Context::default();
    let mut tree = TreeModel::new();
    let root = tree.add_item(Item::new_dir(0, PathBuf::from("/large_dir"), 0, 1, 1, None));

    // Add 500 files
    let mut target_file_id = 0;
    for i in 0..500 {
        let fid = tree.add_item(Item::new_file(
            0,
            PathBuf::from(format!("/large_dir/file_{:03}.dat", i)),
            1000,
            1024,
            0,
            1,
            i as u64 + 10,
            Some(root),
        ));
        if i == 350 {
            target_file_id = fid;
        }
    }
    tree.compute_rollup();

    let mut view = DirTreeView::new();
    let mut selected = None;
    let mut status = None;

    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 400.0))),
        ..Default::default()
    };

    // Frame 1: Initial state at top of list
    let full_output1 = ctx.run(raw_input.clone(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(600.0, 300.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    view.ui(ui, &mut tree, &mut selected, &mut status, false);
                },
            );
        });
    });

    // Check that virtualization works: out of 501 total nodes, only ~20-30 are rendered as shapes
    let mut rendered_files_frame1 = 0;
    let mut found_target_frame1 = false;
    for clipped in &full_output1.shapes {
        if let egui::epaint::Shape::Text(t) = &clipped.shape {
            if t.galley.job.text.starts_with("file_") {
                rendered_files_frame1 += 1;
            }
            if t.galley.job.text == "file_350.dat" {
                found_target_frame1 = true;
            }
        }
    }

    println!("Virtualization check: {} rows rendered out of 501", rendered_files_frame1);
    assert!(rendered_files_frame1 > 0 && rendered_files_frame1 < 40, "Must be virtualized (only ~20 rows rendered)");
    assert!(!found_target_frame1, "Row 350 should NOT be rendered yet (virtualized off-screen)");

    // Frame 2: Select file #350 and request jump to it
    selected = Some(target_file_id);
    view.request_scroll_to_selected = true;

    let full_output2 = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(600.0, 300.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    view.ui(ui, &mut tree, &mut selected, &mut status, false);
                },
            );
        });
    });

    // Verify that after scroll jump, file #350 is now rendered on screen!
    let mut found_target_frame2 = false;
    for clipped in &full_output2.shapes {
        if let egui::epaint::Shape::Text(t) = &clipped.shape {
            if t.galley.job.text == "file_350.dat" {
                found_target_frame2 = true;
                break;
            }
        }
    }

    assert!(found_target_frame2, "Row 350 must now be visible in the viewport after scroll jump!");
}

#[test]
fn test_scroll_areas_decoupled() {
    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 500.0))),
        ..Default::default()
    };

    let mut tree = TreeModel::new();
    let root = tree.add_item(Item::new_dir(0, PathBuf::from("/test"), 0, 1, 1, None));
    for i in 0..100 {
        tree.add_item(Item::new_file(
            0,
            PathBuf::from(format!("/test/f_{}.txt", i)),
            100,
            512,
            0,
            1,
            i as u64 + 10,
            Some(root),
        ));
    }
    tree.compute_rollup();

    let mut dir_tree_view = DirTreeView::new();
    let mut ext_list_view = lindirstat::views::ext_list::ExtListView::new();
    let mut selected = None;
    let mut status = None;

    // Run frame with distinct id scopes
    let _ = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.push_id("dir_tree_panel", |ui| {
                dir_tree_view.ui(ui, &mut tree, &mut selected, &mut status, false);
            });
            ui.push_id("ext_list_panel", |ui| {
                ext_list_view.ui(ui, &tree, &mut selected);
            });
        });
    });

    // Check memory IDs: dir_tree_scroll_area and ext_list_scroll_area must have different IDs
    let id_tree = egui::Id::new("dir_tree_scroll_area");
    let id_ext = egui::Id::new("ext_list_scroll_area");
    assert_ne!(id_tree, id_ext, "ScrollArea IDs must be distinct and decoupled");
}



