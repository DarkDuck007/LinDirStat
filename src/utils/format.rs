// Formatting utilities for LinDirStat, matching WinDirStat output style.

use chrono::{Local, TimeZone};

/// Formats a byte count into human-readable string (e.g. "1.2 GB", "450.3 MB", "12.4 KB", "350 B")
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Formats raw byte count with commas (e.g. 1,048,576)
pub fn format_number(val: u64) -> String {
    let s = val.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

/// Formats percentage (e.g. "45.2%")
pub fn format_percent(fraction: f32) -> String {
    let clamped = fraction.clamp(0.0, 1.0) * 100.0;
    format!("{:.1}%", clamped)
}

/// Formats unix timestamp (seconds) into local date-time string
pub fn format_timestamp(epoch_sec: i64) -> String {
    if epoch_sec <= 0 {
        return "-".to_string();
    }
    match Local.timestamp_opt(epoch_sec, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        chrono::LocalResult::Ambiguous(dt, _) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        chrono::LocalResult::None => "-".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 2), "2.0 GB");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_percent() {
        assert_eq!(format_percent(0.0), "0.0%");
        assert_eq!(format_percent(0.505), "50.5%");
        assert_eq!(format_percent(1.0), "100.0%");
    }
}
