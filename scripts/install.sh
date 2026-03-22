#!/usr/bin/env sh
# Install the tsx CLI — downloads the correct binary for the current platform
# from GitHub Releases and places it in ~/.local/bin (or /usr/local/bin as fallback).
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/ateeq1999/tsx/main/scripts/install.sh | sh
#   # or with a specific version:
#   curl -fsSL .../install.sh | sh -s -- --version v0.2.0

set -e

REPO="ateeq1999/tsx"
BIN_NAME="tsx"
INSTALL_DIR="${TSX_INSTALL_DIR:-}"

# ── Parse flags ───────────────────────────────────────────────────────────────
VERSION=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --dir)     INSTALL_DIR="$2"; shift 2 ;;
    *) echo "Unknown flag: $1" >&2; exit 1 ;;
  esac
done

# ── Detect platform ───────────────────────────────────────────────────────────
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-unknown-linux-gnu"  ;;
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo "Unsupported Linux arch: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-apple-darwin"  ;;
      arm64)   TARGET="aarch64-apple-darwin" ;;
      *) echo "Unsupported macOS arch: $ARCH" >&2; exit 1 ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS. On Windows use: winget install tsx" >&2
    exit 1
    ;;
esac

ARTIFACT="tsx-${TARGET}"

# ── Resolve version ───────────────────────────────────────────────────────────
if [ -z "$VERSION" ]; then
  echo "Fetching latest release..."
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
  if [ -z "$VERSION" ]; then
    echo "Could not determine latest version. Set --version explicitly." >&2
    exit 1
  fi
fi

echo "Installing tsx ${VERSION} (${TARGET})..."

# ── Download ──────────────────────────────────────────────────────────────────
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARTIFACT}"
TMP=$(mktemp)
trap 'rm -f "$TMP"' EXIT

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$URL" -o "$TMP"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP" "$URL"
else
  echo "curl or wget required" >&2; exit 1
fi

chmod +x "$TMP"

# ── Install location ──────────────────────────────────────────────────────────
if [ -z "$INSTALL_DIR" ]; then
  if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
  else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
  fi
fi

DEST="${INSTALL_DIR}/${BIN_NAME}"
mv "$TMP" "$DEST"

echo "tsx installed to ${DEST}"

# ── PATH hint ─────────────────────────────────────────────────────────────────
case ":$PATH:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "Add ${INSTALL_DIR} to your PATH:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac

echo "Run: tsx --version"
