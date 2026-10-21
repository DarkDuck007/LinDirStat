// Scanner status and live progress tracking

use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanStatus {
    Idle,
    Scanning,
    Finished,
    Canceled,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ScanStats {
    pub status: ScanStatus,
    pub root_path: Option<PathBuf>,
    pub current_path: Option<PathBuf>,
    pub files_scanned: usize,
    pub dirs_scanned: usize,
    pub total_bytes: u64,
    pub start_time: Option<Instant>,
    pub elapsed: Duration,
}

impl Default for ScanStats {
    fn default() -> Self {
        Self {
            status: ScanStatus::Idle,
            root_path: None,
            current_path: None,
            files_scanned: 0,
            dirs_scanned: 0,
            total_bytes: 0,
            start_time: None,
            elapsed: Duration::ZERO,
        }
    }
}

impl ScanStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self, root: PathBuf) {
        self.status = ScanStatus::Scanning;
        self.root_path = Some(root);
        self.current_path = None;
        self.files_scanned = 0;
        self.dirs_scanned = 0;
        self.total_bytes = 0;
        self.start_time = Some(Instant::now());
        self.elapsed = Duration::ZERO;
    }

    pub fn update_elapsed(&mut self) {
        if let Some(start) = self.start_time {
            if self.status == ScanStatus::Scanning {
                self.elapsed = start.elapsed();
            }
        }
    }

    pub fn files_per_sec(&self) -> f64 {
        let secs = self.elapsed.as_secs_f64();
        if secs > 0.05 {
            (self.files_scanned + self.dirs_scanned) as f64 / secs
        } else {
            0.0
        }
    }
}

