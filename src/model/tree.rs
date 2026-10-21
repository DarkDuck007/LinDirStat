// Arena-based Directory Tree representing file system hierarchy

use super::extension::ExtensionRegistry;
use super::item::Item;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct TreeModel {
    pub items: Vec<Item>,
    pub root_id: Option<usize>,
    pub extensions: ExtensionRegistry,
    path_to_id: HashMap<PathBuf, usize>,
}

impl TreeModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get(&self, id: usize) -> Option<&Item> {
        self.items.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Item> {
        self.items.get_mut(id)
    }

    pub fn find_by_path(&self, path: &Path) -> Option<usize> {
        self.path_to_id.get(path).copied()
    }

    pub fn add_item(&mut self, mut item: Item) -> usize {
        let id = self.items.len();
        item.id = id;
        self.path_to_id.insert(item.path.clone(), id);

        if let Some(parent_id) = item.parent {
            if let Some(parent) = self.items.get_mut(parent_id) {
                parent.children.push(id);
            }
        } else if self.root_id.is_none() {
            self.root_id = Some(id);
        }

        self.items.push(item);
        id
    }

    /// Recursively computes subtree sizes, items, files, and subdirs from leaves to root
    pub fn compute_rollup(&mut self) {
        if let Some(root) = self.root_id {
            self.rollup_node(root);
            let total_size = self.items[root].size;
            self.extensions = ExtensionRegistry::build_from_items(&self.items, total_size);
        }
    }

    fn rollup_node(&mut self, id: usize) -> (u64, u64, usize, usize, usize) {
        let is_dir = self.items[id].is_dir;
        let children = self.items[id].children.clone();

        if !is_dir || children.is_empty() {
            let item = &self.items[id];
            return (
                item.size,
                item.allocated_size,
                item.items,
                item.files,
                item.subdirs,
            );
        }

        let mut sum_size = 0u64;
        let mut sum_alloc = 0u64;
        let mut sum_items = 1usize; // self
        let mut sum_files = 0usize;
        let mut sum_subdirs = 0usize;

        for child_id in children {
            let (sz, alloc, it, fl, sd) = self.rollup_node(child_id);
            sum_size += sz;
            sum_alloc += alloc;
            sum_items += it;
            sum_files += fl;
            sum_subdirs += sd;
            if self.items[child_id].is_dir {
                sum_subdirs += 1;
            }
        }

        let node = &mut self.items[id];
        node.size = sum_size;
        node.allocated_size = sum_alloc;
        node.items = sum_items;
        node.files = sum_files;
        node.subdirs = sum_subdirs;

        (sum_size, sum_alloc, sum_items, sum_files, sum_subdirs)
    }

    /// Sorts children of all directories by size descending (like WinDirStat)
    pub fn sort_children_by_size(&mut self) {
        let n = self.items.len();
        for i in 0..n {
            if self.items[i].is_dir && !self.items[i].children.is_empty() {
                let mut children = std::mem::take(&mut self.items[i].children);
                let items_ref = &self.items;
                children.sort_by(|&a, &b| {
                    let sz_a = items_ref.get(a).map(|item| item.size).unwrap_or(0);
                    let sz_b = items_ref.get(b).map(|item| item.size).unwrap_or(0);
                    sz_b.cmp(&sz_a)
                });
                self.items[i].children = children;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_rollup() {
        let mut tree = TreeModel::new();
        let root = tree.add_item(Item::new_dir(0, PathBuf::from("/root"), 0, 1, 100, None));
        let sub = tree.add_item(Item::new_dir(0, PathBuf::from("/root/sub"), 0, 1, 101, Some(root)));
        let _f1 = tree.add_item(Item::new_file(0, PathBuf::from("/root/sub/f1.txt"), 100, 512, 0, 1, 102, Some(sub)));
        let _f2 = tree.add_item(Item::new_file(0, PathBuf::from("/root/f2.bin"), 250, 512, 0, 1, 103, Some(root)));

        assert_eq!(tree.items.len(), 4);
        tree.compute_rollup();

        assert_eq!(tree.items[sub].size, 100);
        assert_eq!(tree.items[sub].files, 1);
        assert_eq!(tree.items[sub].subdirs, 0);

        assert_eq!(tree.items[root].size, 350);
        assert_eq!(tree.items[root].allocated_size, 1024);
        assert_eq!(tree.items[root].files, 2);
        assert_eq!(tree.items[root].subdirs, 1);
    }
}
