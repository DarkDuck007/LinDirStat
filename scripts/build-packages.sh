#!/usr/bin/env bash
set -euo pipefail

# Scripts directory
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERSION="${1:-$(grep -m1 '^version' Cargo.toml | awk -F '"' '{print $2}')}"
DIST_DIR="${ROOT_DIR}/dist"
APPDIR="${ROOT_DIR}/target/AppDir"

echo "=== Building LinDirStat v${VERSION} ==="
mkdir -p "$DIST_DIR"
rm -rf "$APPDIR" "$DIST_DIR/*"

# 1. Compile release binary
echo "--> Compiling release binary..."
cargo build --release --locked

BIN_PATH="target/release/lindirstat"
if [ ! -f "$BIN_PATH" ]; then
    echo "Error: Release binary not found at $BIN_PATH"
    exit 1
fi

# Strip binary to reduce size
strip "$BIN_PATH" || true

# 2. Package .tar.gz
TAR_NAME="lindirstat-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz"
echo "--> Creating tar.gz archive: ${TAR_NAME}..."
TEMP_TAR_DIR="$(mktemp -d)"
mkdir -p "${TEMP_TAR_DIR}/lindirstat"
cp "$BIN_PATH" "${TEMP_TAR_DIR}/lindirstat/lindirstat"
cp README.md "${TEMP_TAR_DIR}/lindirstat/README.md"
cp assets/lindirstat.desktop "${TEMP_TAR_DIR}/lindirstat/lindirstat.desktop"
cp assets/lindirstat.png "${TEMP_TAR_DIR}/lindirstat/lindirstat.png"

tar -czf "${DIST_DIR}/${TAR_NAME}" -C "${TEMP_TAR_DIR}" lindirstat
rm -rf "${TEMP_TAR_DIR}"
echo "    Created ${DIST_DIR}/${TAR_NAME}"

# Also create generic alias without version for convenience
cp "${DIST_DIR}/${TAR_NAME}" "${DIST_DIR}/lindirstat-x86_64-unknown-linux-gnu.tar.gz"

# 3. Package .AppImage
echo "--> Preparing AppDir..."
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/metainfo"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"

cp "$BIN_PATH" "${APPDIR}/usr/bin/lindirstat"
cp assets/AppRun "${APPDIR}/AppRun"
chmod +x "${APPDIR}/AppRun"

cp assets/lindirstat.desktop "${APPDIR}/lindirstat.desktop"
cp assets/lindirstat.desktop "${APPDIR}/usr/share/applications/lindirstat.desktop"
cp assets/lindirstat.metainfo.xml "${APPDIR}/usr/share/metainfo/lindirstat.metainfo.xml"
cp assets/lindirstat.png "${APPDIR}/lindirstat.png"
cp assets/lindirstat.png "${APPDIR}/.DirIcon"
cp assets/lindirstat.png "${APPDIR}/usr/share/icons/hicolor/256x256/apps/lindirstat.png"
cp assets/lindirstat.svg "${APPDIR}/usr/share/icons/hicolor/scalable/apps/lindirstat.svg"

# Obtain appimagetool if not present
APPIMAGETOOL="/tmp/appimagetool"
if [ ! -f "$APPIMAGETOOL" ]; then
    echo "--> Downloading appimagetool..."
    curl -sSL -o "$APPIMAGETOOL" "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x "$APPIMAGETOOL"
fi

APPIMAGE_NAME="LinDirStat-v${VERSION}-x86_64.AppImage"
echo "--> Generating AppImage: ${APPIMAGE_NAME}..."
ARCH=x86_64 "$APPIMAGETOOL" --appimage-extract-and-run "$APPDIR" "${DIST_DIR}/${APPIMAGE_NAME}"
cp "${DIST_DIR}/${APPIMAGE_NAME}" "${DIST_DIR}/LinDirStat-x86_64.AppImage"

# 4. Generate SHA256 checksums
echo "--> Generating SHA256 checksums..."
cd "$DIST_DIR"
sha256sum *.tar.gz *.AppImage > SHA256SUMS.txt

echo "=== Release packages successfully created in ${DIST_DIR} ==="
ls -lh "$DIST_DIR"

