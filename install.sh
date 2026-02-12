#!/bin/bash
set -e

# Vibe installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/mzruya/vibe/main/install.sh | bash

REPO="mzruya/vibe"
INSTALL_DIR="$HOME/.vibe/bin"

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin) OS="apple-darwin" ;;
  linux) OS="unknown-linux-gnu" ;;
  *)
    echo "Unsupported OS: $OS"
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

TARGET="${ARCH}-${OS}"

# Get latest release tag
echo "Fetching latest release..."

# Try gh CLI first if available, otherwise use GitHub API
if command -v gh &> /dev/null; then
  LATEST=$(gh release view --repo "$REPO" --json tagName -q '.tagName' 2>/dev/null || true)
fi

# Fall back to GitHub API with redirect following
if [ -z "$LATEST" ]; then
  # Use the releases/latest redirect which doesn't require API auth
  LATEST=$(curl -fsSI "https://github.com/$REPO/releases/latest" 2>/dev/null | grep -i "^location:" | sed 's/.*tag\///' | tr -d '\r\n')
fi

if [ -z "$LATEST" ]; then
  echo "Failed to fetch latest release."
  echo ""
  echo "Install from source instead:"
  echo "  cargo install --git https://github.com/$REPO"
  exit 1
fi

echo "Installing vibe $LATEST for $TARGET..."

# Download and extract
URL="https://github.com/$REPO/releases/download/$LATEST/vibe-$TARGET.tar.gz"
mkdir -p "$INSTALL_DIR"

if ! curl -fsSL "$URL" | tar xz -C "$INSTALL_DIR"; then
  echo ""
  echo "Failed to download release. Install from source instead:"
  echo "  cargo install --git https://github.com/$REPO"
  exit 1
fi

chmod +x "$INSTALL_DIR/vibe"

echo ""
echo "Installed vibe to $INSTALL_DIR/vibe"
echo ""

# Check PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo "Add vibe to your PATH by adding this to your shell profile:"
  echo ""
  echo "  export PATH=\"\$HOME/.vibe/bin:\$PATH\""
  echo ""
fi

echo "Run 'vibe doctor' to verify the installation."
