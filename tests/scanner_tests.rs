use crossbeam_channel::unbounded;
use lindirstat::scanner::worker::{ScanMessage, ScannerOptions, ScannerWorker};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_multilevel_directory_scan() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Create hierarchy:
    // root/
    //   folder_a/
    //     sub_a1/
    //       file1.txt (500 B)
    //     file2.bin (1000 B)
    //   folder_b/
    //     file3.iso (2000 B)
    let folder_a = root.join("folder_a");
    let sub_a1 = folder_a.join("sub_a1");
    let folder_b = root.join("folder_b");

    fs::create_dir_all(&sub_a1).unwrap();
    fs::create_dir_all(&folder_b).unwrap();

    fs::write(sub_a1.join("file1.txt"), vec![0u8; 500]).unwrap();
    fs::write(folder_a.join("file2.bin"), vec![0u8; 1000]).unwrap();
    fs::write(folder_b.join("file3.iso"), vec![0u8; 2000]).unwrap();

    let (tx, rx) = unbounded();
    let cancel = Arc::new(AtomicBool::new(false));
    let worker = ScannerWorker::new(ScannerOptions::default(), cancel);

    worker.run(root.to_path_buf(), tx);

    let mut finished_tree = None;
    while let Ok(msg) = rx.recv() {
        if let ScanMessage::Finished(tree) = msg {
            finished_tree = Some(tree);
            break;
        }
    }

    let tree = finished_tree.expect("Expected scan to finish");
    let root_item = tree.get(tree.root_id.unwrap()).unwrap();

    assert_eq!(root_item.files, 3);
    assert_eq!(root_item.subdirs, 3); // folder_a, sub_a1, folder_b
    assert_eq!(root_item.size, 3500);
}

#[test]
fn test_hardlink_deduplication() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let original = root.join("original.bin");
    let hardlink = root.join("hardlink.bin");

    // Create 100 KB file
    fs::write(&original, vec![0xAA; 100 * 1024]).unwrap();

    // Create hard link to same inode
    fs::hard_link(&original, &hardlink).unwrap();

    let (tx, rx) = unbounded();
    let cancel = Arc::new(AtomicBool::new(false));
    let worker = ScannerWorker::new(ScannerOptions::default(), cancel);

    worker.run(root.to_path_buf(), tx);

    let mut finished_tree = None;
    while let Ok(msg) = rx.recv() {
        if let ScanMessage::Finished(tree) = msg {
            finished_tree = Some(tree);
            break;
        }
    }

    let tree = finished_tree.expect("Expected scan to finish");
    let root_item = tree.get(tree.root_id.unwrap()).unwrap();

    // 2 files in directory, but hardlinked size should only be counted once (100 KB, not 200 KB)!
    assert_eq!(root_item.files, 2);
    assert_eq!(root_item.size, 100 * 1024);
}

#[test]
fn test_symlink_safety_and_no_loop() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let dir1 = root.join("dir1");
    fs::create_dir(&dir1).unwrap();
    fs::write(dir1.join("dummy.txt"), "hello").unwrap();

    // Create symlink pointing back to root (would infinite loop if followed)
    let symlink_to_root = dir1.join("loop_link");
    unix_fs::symlink(root, &symlink_to_root).unwrap();

    let (tx, rx) = unbounded();
    let cancel = Arc::new(AtomicBool::new(false));
    let options = ScannerOptions {
        follow_symlinks: false,
        ..Default::default()
    };
    let worker = ScannerWorker::new(options, cancel);

    worker.run(root.to_path_buf(), tx);

    let mut finished_tree = None;
    while let Ok(msg) = rx.recv() {
        if let ScanMessage::Finished(tree) = msg {
            finished_tree = Some(tree);
            break;
        }
    }

    let tree = finished_tree.expect("Expected scan to finish safely without looping");
    let root_item = tree.get(tree.root_id.unwrap()).unwrap();
    // 1 regular file + 1 symlink file (not recursed into) = 2 files total
    assert_eq!(root_item.files, 2);
    assert_eq!(root_item.subdirs, 1);
}

#[test]
fn test_scanner_cancellation() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    for i in 0..50 {
        let d = root.join(format!("dir_{}", i));
        fs::create_dir(&d).unwrap();
        fs::write(d.join("file.txt"), vec![0u8; 100]).unwrap();
    }

    let (tx, rx) = unbounded();
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = Arc::clone(&cancel);

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1));
        cancel_clone.store(true, Ordering::Relaxed);
    });

    let worker = ScannerWorker::new(ScannerOptions::default(), cancel);
    worker.run(root.to_path_buf(), tx);

    let mut got_canceled_or_finished = false;
    while let Ok(msg) = rx.recv() {
        match msg {
            ScanMessage::Canceled | ScanMessage::Finished(_) => {
                got_canceled_or_finished = true;
                break;
            }
            _ => {}
        }
    }

    assert!(got_canceled_or_finished);
}
