// Linux-specific filesystem inspection and kernel filesystem detection (ncdu reference)

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct LinuxFileInfo {
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub allocated_size: u64,
    pub mtime: i64,
    pub dev: u64,
    pub ino: u64,
    pub nlink: u64,
}

/// Reads Linux filesystem metadata without following symlinks
pub fn read_linux_metadata(path: &Path) -> std::io::Result<LinuxFileInfo> {
    let meta = std::fs::symlink_metadata(path)?;
    let is_symlink = meta.file_type().is_symlink();
    let is_dir = meta.is_dir();

    // On Linux, blocks are 512 bytes each
    let allocated_size = meta.blocks() * 512;

    Ok(LinuxFileInfo {
        is_dir,
        is_symlink,
        size: meta.len(),
        allocated_size,
        mtime: meta.mtime(),
        dev: meta.dev(),
        ino: meta.ino(),
        nlink: meta.nlink(),
    })
}

/// Checks if a directory is a virtual kernel filesystem (/proc, /sys, /sys/kernel/debug, cgroup, etc.)
pub fn is_kernel_fs(path: &Path) -> bool {
    let Ok(c_path) = CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };

    unsafe {
        let mut buf: libc::statfs = std::mem::zeroed();
        if libc::statfs(c_path.as_ptr(), &mut buf) != 0 {
            return false;
        }

        // Standard Linux magic numbers (statfs(2))
        matches!(
            buf.f_type as u64,
            0x42494E4D   // BINFMTFS_MAGIC
            | 0xCAFE4A11 // BPF_FS_MAGIC
            | 0x0027E0EB // CGROUP_SUPER_MAGIC
            | 0x63677270 // CGROUP2_SUPER_MAGIC
            | 0x64626720 // DEBUGFS_MAGIC
            | 0x00001CD1 // DEVPTS_SUPER_MAGIC
            | 0x00009FA0 // PROC_SUPER_MAGIC
            | 0x6165676C // PSTOREFS_MAGIC
            | 0x73636673 // SECURITYFS_MAGIC
            | 0xF97CFF8C // SELINUX_MAGIC
            | 0x62656572 // SYSFS_MAGIC
            | 0x74726163 // TRACEFS_MAGIC
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_linux_metadata() {
        let tmp = std::env::temp_dir();
        let meta = read_linux_metadata(&tmp).expect("Failed to read temp dir metadata");
        assert!(meta.is_dir);
        assert!(!meta.is_symlink);
        assert!(meta.dev > 0);
    }

    #[test]
    fn test_is_kernel_fs() {
        assert!(is_kernel_fs(Path::new("/proc")));
        assert!(is_kernel_fs(Path::new("/sys")));
        // /tmp or /home should not be kernel fs
        let tmp = std::env::temp_dir();
        assert!(!is_kernel_fs(&tmp));
    }
}
