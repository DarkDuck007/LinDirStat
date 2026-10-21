// High-performance Linux directory scanner with live progress streaming

use crossbeam_channel::Sender;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use super::linux::{is_kernel_fs, read_linux_metadata};
use crate::model::item::Item;
use crate::model::tree::TreeModel;

#[derive(Debug, Clone)]
pub struct ScannerOptions {
    pub same_fs: bool,
    pub exclude_kernel_fs: bool,
    pub follow_symlinks: bool,
}

impl Default for ScannerOptions {
    fn default() -> Self {
        Self {
            same_fs: false,
            exclude_kernel_fs: true,
            follow_symlinks: false,
        }
    }
}

pub enum ScanMessage {
    Progress {
        current_path: PathBuf,
        files_count: usize,
        dirs_count: usize,
        bytes_count: u64,
    },
    Finished(TreeModel),
    Canceled,
    Error(String),
}

pub struct ScannerWorker {
    options: ScannerOptions,
    cancel_flag: Arc<AtomicBool>,
}

impl ScannerWorker {
    pub fn new(options: ScannerOptions, cancel_flag: Arc<AtomicBool>) -> Self {
        Self {
            options,
            cancel_flag,
        }
    }

    /// Spawns a background thread to scan the root path and send updates
    pub fn scan_async(
        root_path: PathBuf,
        options: ScannerOptions,
        cancel_flag: Arc<AtomicBool>,
        tx: Sender<ScanMessage>,
    ) {
        std::thread::Builder::new()
            .name("lindirstat-scanner".to_string())
            .spawn(move || {
                let worker = ScannerWorker::new(options, cancel_flag);
                worker.run(root_path, tx);
            })
            .expect("Failed to spawn scanner thread");
    }

    pub fn run(&self, root_path: PathBuf, tx: Sender<ScanMessage>) {
        let meta = match read_linux_metadata(&root_path) {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(ScanMessage::Error(format!(
                    "Cannot access {}: {}",
                    root_path.display(),
                    e
                )));
                return;
            }
        };

        let root_dev = meta.dev;
        let mut tree = TreeModel::new();
        let mut seen_inodes: HashSet<(u64, u64)> = HashSet::new();

        let root_id = tree.add_item(Item::new_dir(
            0,
            root_path.clone(),
            meta.mtime,
            meta.dev,
            meta.ino,
            None,
        ));

        let mut files_scanned = 0usize;
        let mut dirs_scanned = 1usize;
        let mut bytes_scanned = 0u64;

        let mut last_progress_time = Instant::now();

        // Stack-based depth-first or breadth traversal for directories
        let mut dir_stack: Vec<(PathBuf, usize)> = vec![(root_path, root_id)];

        while let Some((dir_path, dir_id)) = dir_stack.pop() {
            if self.cancel_flag.load(Ordering::Relaxed) {
                let _ = tx.send(ScanMessage::Canceled);
                return;
            }

            // Read directory entries
            let entries = match std::fs::read_dir(&dir_path) {
                Ok(rd) => rd,
                Err(_) => {
                    // Permission denied or unreadable directory, continue
                    continue;
                }
            };

            for entry in entries.flatten() {
                if self.cancel_flag.load(Ordering::Relaxed) {
                    let _ = tx.send(ScanMessage::Canceled);
                    return;
                }

                let child_path = entry.path();
                let child_meta = match read_linux_metadata(&child_path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                // Respect same filesystem constraint if enabled
                if self.options.same_fs && child_meta.dev != root_dev {
                    continue;
                }

                // Skip kernel virtual filesystems
                if self.options.exclude_kernel_fs && child_meta.is_dir && is_kernel_fs(&child_path)
                {
                    continue;
                }

                if child_meta.is_dir && (!child_meta.is_symlink || self.options.follow_symlinks) {
                    dirs_scanned += 1;
                    let child_id = tree.add_item(Item::new_dir(
                        0,
                        child_path.clone(),
                        child_meta.mtime,
                        child_meta.dev,
                        child_meta.ino,
                        Some(dir_id),
                    ));
                    dir_stack.push((child_path.clone(), child_id));
                } else if !child_meta.is_dir {
                    // File entry: check hardlink deduplication
                    let mut size = child_meta.size;
                    let mut alloc = child_meta.allocated_size;

                    if child_meta.nlink > 1 {
                        let key = (child_meta.dev, child_meta.ino);
                        if seen_inodes.contains(&key) {
                            // Deduplicate: already counted elsewhere
                            size = 0;
                            alloc = 0;
                        } else {
                            seen_inodes.insert(key);
                        }
                    }

                    files_scanned += 1;
                    bytes_scanned += size;

                    tree.add_item(Item::new_file(
                        0,
                        child_path.clone(),
                        size,
                        alloc,
                        child_meta.mtime,
                        child_meta.dev,
                        child_meta.ino,
                        Some(dir_id),
                    ));
                }

                // Send throttled UI progress updates (~30 times/sec)
                if last_progress_time.elapsed().as_millis() > 33 {
                    last_progress_time = Instant::now();
                    let _ = tx.send(ScanMessage::Progress {
                        current_path: child_path,
                        files_count: files_scanned,
                        dirs_count: dirs_scanned,
                        bytes_count: bytes_scanned,
                    });
                }
            }
        }

        // Finalize: compute rolled-up sizes and extension statistics
        tree.compute_rollup();
        tree.sort_children_by_size();

        let _ = tx.send(ScanMessage::Finished(tree));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_worker_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("test.txt"), "hello world 12345").unwrap();
        std::fs::write(tmp.path().join("root.bin"), vec![0u8; 100]).unwrap();

        let (tx, rx) = crossbeam_channel::unbounded();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker = ScannerWorker::new(ScannerOptions::default(), cancel);
        worker.run(tmp.path().to_path_buf(), tx);

        let mut finished_tree = None;
        while let Ok(msg) = rx.recv() {
            if let ScanMessage::Finished(tree) = msg {
                finished_tree = Some(tree);
                break;
            }
        }

        assert!(finished_tree.is_some());
        let tree = finished_tree.unwrap();
        assert_eq!(tree.items.len(), 4); // root, sub, test.txt, root.bin
        let root = &tree.items[tree.root_id.unwrap()];
        assert_eq!(root.files, 2);
        assert_eq!(root.subdirs, 1);
        assert!(root.size >= 115);
    }
}
