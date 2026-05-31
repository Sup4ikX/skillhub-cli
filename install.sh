#!/bin/bash
set -euo pipefail

REPO="Sup4ikX/skillhub-cli"
VERSION="${1:-latest}"
INSTALL_DIR="${2:-$HOME/.skillhub/bin}"

echo "Installing skillhub..."

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux)  TARGET="x86_64-unknown-linux-gnu" ;;
  darwin)
    case "$ARCH" in
      arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
      *)             TARGET="x86_64-apple-darwin" ;;
    esac
    ;;
  *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  aarch64|arm64)
    case "$OS" in
      linux) TARGET="aarch64-unknown-linux-gnu" ;;
    esac
    ;;
esac

# Fetch latest version if not specified
if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" |
    grep '"tag_name"' | sed 's/.*"tag_name": "\(.*\)".*/\1/')
  if [ -z "$VERSION" ]; then
    echo "Failed to fetch latest version. Try specifying a version manually."
    echo "Usage: curl -fsSL https://skillhub.sh/install.sh | bash -s v0.1.0"
    exit 1
  fi
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/skillhub-$TARGET.tar.gz"

echo "  Platform:  $TARGET"
echo "  Version:   $VERSION"
echo "  Download:  $DOWNLOAD_URL"
echo "  Install:   $INSTALL_DIR"
echo

mkdir -p "$INSTALL_DIR"

# Download and extract
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/skillhub.tar.gz"
tar xzf "$TMP_DIR/skillhub.tar.gz" -C "$TMP_DIR"
install "$TMP_DIR/skillhub" "$INSTALL_DIR/skillhub"

echo "Installed skillhub to $INSTALL_DIR/skillhub"
echo

# Add to PATH
case "$SHELL" in
  */bash|*/zsh)
    RC_FILE="$HOME/.${SHELL##*/}rc"
    if ! grep -q "skillhub/bin" "$RC_FILE" 2>/dev/null; then
      echo >> "$RC_FILE"
      echo "# skillhub" >> "$RC_FILE"
      echo "export PATH=\"\$HOME/.skillhub/bin:\$PATH\"" >> "$RC_FILE"
      echo "Added \$HOME/.skillhub/bin to PATH in $RC_FILE"
      echo "Run: source $RC_FILE"
    else
      echo "PATH already configured in $RC_FILE"
    fi
    ;;
  *)
    echo "Add to PATH manually: export PATH=\"\$HOME/.skillhub/bin:\$PATH\""
    ;;
esac

echo
echo "Run 'skillhub setup' to configure your GitHub token."
