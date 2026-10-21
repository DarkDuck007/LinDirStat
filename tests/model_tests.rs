use lindirstat::model::item::{extract_extension, Item};
use lindirstat::model::tree::TreeModel;
use std::path::PathBuf;

#[test]
fn test_complex_extensions_parsing() {
    assert_eq!(extract_extension("package.tar.gz").as_deref(), Some(".tar.gz"));
    assert_eq!(extract_extension("archive.tar.bz2").as_deref(), Some(".tar.bz2"));
    assert_eq!(extract_extension("ARCHIVE.TAR.XZ").as_deref(), Some(".tar.xz"));
    assert_eq!(extract_extension("document.PDF").as_deref(), Some(".pdf"));
    assert_eq!(extract_extension("script.sh").as_deref(), Some(".sh"));
    assert_eq!(extract_extension("README").as_deref(), None);
    assert_eq!(extract_extension(".gitignore").as_deref(), None);
}

#[test]
fn test_deep_tree_rollup_and_sorting() {
    let mut tree = TreeModel::new();

    // /root
    //   /dir1 (sum: 300)
    //     /dir2 (sum: 200)
    //       file_deep.bin (200 B)
    //     file_shallow.txt (100 B)
    //   /dir_large (sum: 700)
    //     file_big.iso (700 B)
    let root = tree.add_item(Item::new_dir(0, PathBuf::from("/root"), 0, 1, 1, None));
    let dir1 = tree.add_item(Item::new_dir(0, PathBuf::from("/root/dir1"), 0, 1, 2, Some(root)));
    let dir2 = tree.add_item(Item::new_dir(0, PathBuf::from("/root/dir1/dir2"), 0, 1, 3, Some(dir1)));
    tree.add_item(Item::new_file(0, PathBuf::from("/root/dir1/dir2/file_deep.bin"), 200, 512, 0, 1, 4, Some(dir2)));
    tree.add_item(Item::new_file(0, PathBuf::from("/root/dir1/file_shallow.txt"), 100, 512, 0, 1, 5, Some(dir1)));

    let dir_large = tree.add_item(Item::new_dir(0, PathBuf::from("/root/dir_large"), 0, 1, 6, Some(root)));
    tree.add_item(Item::new_file(0, PathBuf::from("/root/dir_large/file_big.iso"), 700, 1024, 0, 1, 7, Some(dir_large)));

    tree.compute_rollup();

    assert_eq!(tree.items[dir2].size, 200);
    assert_eq!(tree.items[dir1].size, 300);
    assert_eq!(tree.items[dir_large].size, 700);
    assert_eq!(tree.items[root].size, 1000);
    assert_eq!(tree.items[root].files, 3);
    assert_eq!(tree.items[root].subdirs, 3);

    // Test children sorting: root's children should be [dir_large (700), dir1 (300)]
    tree.sort_children_by_size();
    assert_eq!(tree.items[root].children, vec![dir_large, dir1]);
}

#[test]
fn test_extension_ranking_and_colors() {
    let mut tree = TreeModel::new();
    let root = tree.add_item(Item::new_dir(0, PathBuf::from("/root"), 0, 1, 1, None));

    // 500 MB of .mp4, 300 MB of .zip, 200 MB of .rs
    tree.add_item(Item::new_file(0, PathBuf::from("/root/vid1.mp4"), 300_000_000, 0, 0, 1, 2, Some(root)));
    tree.add_item(Item::new_file(0, PathBuf::from("/root/vid2.mp4"), 200_000_000, 0, 0, 1, 3, Some(root)));
    tree.add_item(Item::new_file(0, PathBuf::from("/root/pack.zip"), 300_000_000, 0, 0, 1, 4, Some(root)));
    tree.add_item(Item::new_file(0, PathBuf::from("/root/code.rs"), 200_000_000, 0, 0, 1, 5, Some(root)));

    tree.compute_rollup();
    let reg = &tree.extensions;

    assert_eq!(reg.extensions.len(), 3);
    assert_eq!(reg.extensions[0].extension, ".mp4");
    assert_eq!(reg.extensions[0].total_bytes, 500_000_000);
    assert_eq!(reg.extensions[0].file_count, 2);
    assert!((reg.extensions[0].percentage - 0.50).abs() < 0.01);

    assert_eq!(reg.extensions[1].extension, ".zip");
    assert_eq!(reg.extensions[1].total_bytes, 300_000_000);

    assert_eq!(reg.extensions[2].extension, ".rs");
    assert_eq!(reg.extensions[2].total_bytes, 200_000_000);

    // Verify each extension received a distinct color
    let c_mp4 = reg.get_color_for_ext(Some(".mp4"));
    let c_zip = reg.get_color_for_ext(Some(".zip"));
    let c_rs = reg.get_color_for_ext(Some(".rs"));
    assert_ne!(c_mp4, c_zip);
    assert_ne!(c_zip, c_rs);
}
