#!/usr/bin/env bash
# Bellwether installer script.
#
#   curl -fsSL https://raw.githubusercontent.com/<sheepy-user1>/bellwether/main/scripts/install.sh | bash
#
# Downloads the latest prebuilt Linux x86_64 binary from GitHub Releases and
# drops it into ~/.local/bin. Falls back to building from source with cargo
# if no matching release asset is found (e.g. you're on a fresh commit that
# hasn't been released yet).
set -euo pipefail

REPO="${BELLWETHER_REPO:-sheepy-user1/bellwether}"
INSTALL_DIR="${BELLWETHER_INSTALL_DIR:-$HOME/.local/bin}"
BIN_NAME="bellwether"

say() { printf '\033[1;33m[bellwether]\033[0m %s\n' "$1"; }
fail() { printf '\033[1;31m[bellwether]\033[0m %s\n' "$1" >&2; exit 1; }

mkdir -p "$INSTALL_DIR"

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64) ASSET="bellwether-linux-x86_64" ;;
  aarch64|arm64) ASSET="bellwether-linux-aarch64" ;;
  *) fail "unsupported architecture: $ARCH" ;;
esac

LATEST_URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

say "trying prebuilt release binary for ${ARCH}..."
if curl -fsSL -o "$INSTALL_DIR/$BIN_NAME" "$LATEST_URL" 2>/dev/null; then
  chmod +x "$INSTALL_DIR/$BIN_NAME"
  say "installed to $INSTALL_DIR/$BIN_NAME"
else
  say "no release asset found, building from source instead (needs cargo)..."
  command -v cargo >/dev/null 2>&1 || fail "cargo not found; install Rust first: https://www.rust-lang.org/tools/install"
  TMP_DIR="$(mktemp -d)"
  trap 'rm -rf "$TMP_DIR"' EXIT
  git clone --depth 1 "https://github.com/${REPO}.git" "$TMP_DIR"
  (cd "$TMP_DIR" && cargo build --release -p bellwether-cli)
  cp "$TMP_DIR/target/release/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
  say "built and installed to $INSTALL_DIR/$BIN_NAME"
fi

case ":$PATH:" in
  *":$INSTALL_DIR:"*) : ;;
  *) say "note: $INSTALL_DIR is not on your PATH. Add this to your shell rc:" ; echo "  export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
esac

say "run 'bellwether tui' for the interactive checklist, or 'bellwether list' to see all apps."
