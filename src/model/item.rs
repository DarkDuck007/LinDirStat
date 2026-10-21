// File and directory item representation

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Item {
    pub id: usize,
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub allocated_size: u64,
    pub mtime: i64,
    pub items: usize,
    pub files: usize,
    pub subdirs: usize,
    pub dev: u64,
    pub ino: u64,
    pub extension: Option<String>,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub expanded: bool,
}

impl Item {
    pub fn new_file(
        id: usize,
        path: PathBuf,
        size: u64,
        allocated_size: u64,
        mtime: i64,
        dev: u64,
        ino: u64,
        parent: Option<usize>,
    ) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        let extension = extract_extension(&name);

        Self {
            id,
            name,
            path,
            is_dir: false,
            size,
            allocated_size,
            mtime,
            items: 1,
            files: 1,
            subdirs: 0,
            dev,
            ino,
            extension,
            children: Vec::new(),
            parent,
            expanded: false,
        }
    }

    pub fn new_dir(
        id: usize,
        path: PathBuf,
        mtime: i64,
        dev: u64,
        ino: u64,
        parent: Option<usize>,
    ) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        Self {
            id,
            name,
            path,
            is_dir: true,
            size: 0,
            allocated_size: 0,
            mtime,
            items: 1,
            files: 0,
            subdirs: 0,
            dev,
            ino,
            extension: None,
            children: Vec::new(),
            parent,
            expanded: false,
        }
    }
}

/// Extracts lowercase file extension, handling special Linux multi-part extensions like .tar.gz
pub fn extract_extension(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    if lower.ends_with(".tar.gz") {
        return Some(".tar.gz".to_string());
    }
    if lower.ends_with(".tar.bz2") {
        return Some(".tar.bz2".to_string());
    }
    if lower.ends_with(".tar.xz") {
        return Some(".tar.xz".to_string());
    }

    let p = Path::new(&lower);
    p.extension().map(|e| {
        let mut s = ".".to_string();
        s.push_str(&e.to_string_lossy());
        s
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_extension() {
        assert_eq!(extract_extension("movie.MP4").as_deref(), Some(".mp4"));
        assert_eq!(extract_extension("archive.tar.gz").as_deref(), Some(".tar.gz"));
        assert_eq!(extract_extension("no_ext"), None);
        assert_eq!(extract_extension(".hidden"), None);
    }
}

