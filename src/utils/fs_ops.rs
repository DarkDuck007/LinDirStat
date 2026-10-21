// Linux desktop filesystem operations: opening files, opening in file manager, terminal, trash

use std::path::Path;
use std::process::Command;

/// Opens the file or directory using the default Linux application (xdg-open / open crate)
pub fn open_file(path: &Path) -> Result<(), String> {
    open::that(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))
}

/// Reveals the item in the desktop file manager (nautilus, dolphin, thunar, pcmanfm, or xdg-open)
pub fn reveal_in_file_manager(path: &Path) -> Result<(), String> {
    let parent = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    // Try xdg-open on parent folder
    open::that(parent).map_err(|e| format!("Failed to open directory {}: {}", parent.display(), e))
}

/// Opens a Linux terminal emulator in the folder
pub fn open_terminal(path: &Path) -> Result<(), String> {
    let dir = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    let terminals = [
        ("x-terminal-emulator", vec!["--working-directory"]),
        ("gnome-terminal", vec!["--working-directory"]),
        ("konsole", vec!["--workdir"]),
        ("xfce4-terminal", vec!["--working-directory"]),
        ("alacritty", vec!["--working-directory"]),
        ("kitty", vec!["--directory"]),
        ("xterm", vec![]),
    ];

    for (term, args) in terminals {
        let mut cmd = Command::new(term);
        if !args.is_empty() {
            cmd.arg(&args[0]).arg(dir);
        } else {
            cmd.current_dir(dir);
        }
        if cmd.spawn().is_ok() {
            return Ok(());
        }
    }

    Err("No compatible terminal emulator found".to_string())
}

/// Moves a file or directory to the system trash
pub fn move_to_trash(path: &Path) -> Result<(), String> {
    trash::delete(path).map_err(|e| format!("Failed to trash {}: {}", path.display(), e))
}

/// Permanently removes a file or directory
pub fn delete_permanently(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| format!("Failed to delete dir: {}", e))
    } else {
        std::fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))
    }
}
