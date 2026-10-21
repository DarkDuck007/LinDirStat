# LinDirStat

A near 1-1 clone of **WinDirStat** for Linux, written in **Rust** using the **egui** immediate-mode GUI library.

LinDirStat incorporates high-performance Linux directory traversal techniques inspired by **ncdu** and reproduces WinDirStat's iconic 3-pane layout, 3D cushion treemap visualization, extension ranking palette, and chomping Pacman animation.

---

## 🌟 Key Features

1. **Top-Left: Directory Tree View**
   - Hierarchical tree with expandable/collapsible folders and distinct directory (`📁`) and file (`📄`) icons.
   - Graphical Subtree Percentage bar (`Subtree %`).
   - Columns: **Name**, **Subtree %**, **Size**, **Allocated Size** (actual 512-byte blocks on Linux), **Items**, **Files**, **Subdirs**, and **Last Change** timestamp.
   - Interactive sorting: Click column headers to sort by Size, Name, Items, Files, Subdirs, or Date.
   - Selection synchronization: Clicking any folder or file highlights it immediately in the Treemap.

2. **Top-Right: Extension List & Legend**
   - WinDirStat's classic palette ranking by aggregate size descending.
   - Columns: **Extension** (e.g. `.zip`, `.mp4`, `.tar.gz`, `<Files>`), **Color swatch**, **Bytes (%)**, **Bytes (formatted)**, and **Files count**.
   - Selecting an extension highlights and navigates to the largest file of that extension.

3. **Bottom: Squarified Cushion Treemap Visualizer**
   - **Squarified algorithm**: Partitions rectangles by size while minimizing extreme aspect ratios.
   - **Van Wijk 3D Cushion Shading**: Multi-level parabolic ridge calculation creating a cushioned/pillowed appearance that visually conveys directory hierarchy boundaries.
   - **Color-coded per extension** matching the extension legend.
   - **Interactive selection**: Clicking any cushion rectangle selects the file in the Directory Tree, scrolls to it, and highlights the rectangle with WinDirStat's classic bright bounding frame.
   - **Hover tooltips**: Displays file name, size, type, and full path under cursor.
   - **Zooming**: Right-click or toolbar "Zoom In" to make any subdirectory the temporary root, and "Zoom Out" to reset.
   - **Shading toggles**: Toggle between 3D Cushion shading and flat colors, and toggle grid borders.

4. **High-Performance Linux Traversal (ncdu-inspired)**
   - Multi-threaded scanning with work queue.
   - Linux metadata querying (`fstatat`, `dev`, `ino`, `blocks`).
   - Hard-link deduplication: Hardlinked files sharing `(dev, ino)` are only counted once in byte size, avoiding inflated disk metrics.
   - Kernel filesystem exclusion: Automatically identifies and avoids kernel virtual mounts (`/proc`, `/sys`, `devpts`, `cgroup`, etc.).
   - Symlink loop safety: Avoids infinite cycles from circular directory links.
   - Same-filesystem option: Optional flag to avoid crossing filesystem / mount boundaries.

5. **Iconic WinDirStat Chomping Pacman**
   - Animated Pacman chomps yellow pellets across the status bar while a scan is in progress.

6. **Linux Desktop Integration**
   - Move to Trash (`trash` support).
   - Reveal in Linux File Manager (Nautilus, Dolphin, Thunar, etc.).
   - Open in default Linux application (`xdg-open`).
   - Launch Linux Terminal emulator at selected directory.
   - Copy path to clipboard.

---

## 🚀 Getting Started

### Prerequisites

On Debian/Ubuntu:
```bash
sudo apt-get install -y build-essential pkg-config libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgl1-mesa-dev libegl1-mesa-dev libfontconfig1-dev
```

### Building & Running

Run with default GUI:
```bash
cargo run --release
```

Scan a specific directory directly on launch:
```bash
cargo run --release -- /home/user/Downloads
```

### Running Autonomous Tests

Execute unit and integration tests:
```bash
cargo test
```

---

## ⌨️ Shortcuts

- **Ctrl + O**: Open folder picker
- **F5**: Refresh / Rescan current directory
- **Delete**: Move selected item to Trash
- **Left Click**: Select file/folder and synchronize across views
- **Double Click (Tree)**: Expand or collapse folder
- **Right Click (Tree or Treemap)**: Context menu (Open, Reveal, Terminal, Copy Path, Zoom, Trash)

