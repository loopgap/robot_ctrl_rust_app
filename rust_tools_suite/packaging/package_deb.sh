#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VERSION=""
SKIP_BUILD=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="$2"
      shift 2
      ;;
    --skip-build)
      SKIP_BUILD=1
      shift
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "$VERSION" ]]; then
  VERSION="$(grep -E '^version = "[0-9]+\.[0-9]+\.[0-9]+"' "$ROOT_DIR/robot_control_rust/Cargo.toml" | head -n1 | sed -E 's/version = "([^"]+)"/\1/')"
fi

if [[ -z "$VERSION" ]]; then
  echo "Unable to determine package version" >&2
  exit 1
fi

if ! command -v mdbook >/dev/null 2>&1; then
  echo "mdbook is required to package local documentation" >&2
  exit 1
fi

if [[ "$SKIP_BUILD" -ne 1 ]]; then
  cargo build --release --manifest-path "$ROOT_DIR/robot_control_rust/Cargo.toml"
  cargo build --release --manifest-path "$ROOT_DIR/rust_tools_suite/Cargo.toml"
fi

OUT_DIR="$ROOT_DIR/dist/debian"
PKG_NAME="rust-tools-suite"
ARCH="amd64"
STAGE_DIR="$OUT_DIR/${PKG_NAME}_${VERSION}_${ARCH}"
PKG_FILE="$OUT_DIR/${PKG_NAME}_${VERSION}_${ARCH}.deb"

rm -rf "$STAGE_DIR" "$PKG_FILE"
mkdir -p \
  "$STAGE_DIR/DEBIAN" \
  "$STAGE_DIR/usr/bin" \
  "$STAGE_DIR/usr/share/applications" \
  "$STAGE_DIR/usr/share/icons/hicolor/scalable/apps" \
  "$STAGE_DIR/usr/share/rust-tools-suite/docs"

cp "$ROOT_DIR/robot_control_rust/target/release/robot_control_rust" "$STAGE_DIR/usr/bin/"
cp "$ROOT_DIR/rust_tools_suite/target/release/rust_tools_suite" "$STAGE_DIR/usr/bin/"
cp "$ROOT_DIR/rust_tools_suite/packaging/linux/robot-control-rust.desktop" "$STAGE_DIR/usr/share/applications/"
cp "$ROOT_DIR/rust_tools_suite/packaging/linux/rust-tools-suite.desktop" "$STAGE_DIR/usr/share/applications/"
cp "$ROOT_DIR/rust_tools_suite/packaging/linux/robot-control-rust.svg" "$STAGE_DIR/usr/share/icons/hicolor/scalable/apps/"
cp "$ROOT_DIR/rust_tools_suite/packaging/linux/rust-tools-suite.svg" "$STAGE_DIR/usr/share/icons/hicolor/scalable/apps/"
cp "$ROOT_DIR/docs/help/index.html" "$STAGE_DIR/usr/share/rust-tools-suite/help_index.html"
cp "$ROOT_DIR/docs/help/index.html" "$STAGE_DIR/usr/share/rust-tools-suite/docs/index.html"
mdbook build "$ROOT_DIR/docs" -d "$STAGE_DIR/usr/share/rust-tools-suite/docs/book"

cat > "$STAGE_DIR/DEBIAN/control" <<EOF
Package: ${PKG_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: loopgap
Depends: libc6, libgcc-s1, libstdc++6, libgtk-3-0, libudev1
Description: Rust desktop tool bundle for robot control and utility workflows
 Includes robot_control_rust and rust_tools_suite desktop applications,
 responsive layouts, Chinese font fallback guidance, and bundled local help docs.
EOF

dpkg-deb --build --root-owner-group "$STAGE_DIR" "$PKG_FILE" >/dev/null
echo "$PKG_FILE"
