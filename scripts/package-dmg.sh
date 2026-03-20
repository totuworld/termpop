#!/bin/bash
set -euo pipefail

# TermPop .app bundle + DMG packaging script
# Usage:
#   ./scripts/package-dmg.sh [--version 0.1.0] [--binary path/to/termpop] [--sign "Developer ID Application: ..."] [--notarize]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Defaults
VERSION=""
BINARY=""
SIGN_IDENTITY=""
NOTARIZE=false
ARCH=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --version) VERSION="$2"; shift 2 ;;
    --binary) BINARY="$2"; shift 2 ;;
    --sign) SIGN_IDENTITY="$2"; shift 2 ;;
    --notarize) NOTARIZE=true; shift ;;
    --arch) ARCH="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# Auto-detect version from Cargo.toml
if [[ -z "$VERSION" ]]; then
  VERSION=$(grep '^version' "$PROJECT_DIR/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
fi

# Auto-detect binary
if [[ -z "$BINARY" ]]; then
  BINARY="$PROJECT_DIR/target/release/termpop"
fi

# Auto-detect arch
if [[ -z "$ARCH" ]]; then
  ARCH=$(file "$BINARY" | grep -o 'arm64\|x86_64' | head -1)
fi

echo "==> Packaging TermPop v${VERSION} (${ARCH})"

# Paths
APP_NAME="TermPop.app"
BUILD_DIR="$PROJECT_DIR/target/dmg-build"
APP_DIR="$BUILD_DIR/$APP_NAME"
DMG_NAME="TermPop-v${VERSION}-macos-${ARCH}.dmg"
DMG_PATH="$PROJECT_DIR/target/$DMG_NAME"

# Clean
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# --- 1. Create .app bundle ---
echo "==> Creating .app bundle"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# Copy binary
cp "$BINARY" "$APP_DIR/Contents/MacOS/termpop"
chmod +x "$APP_DIR/Contents/MacOS/termpop"

# Copy Info.plist with version substitution
sed "s/__VERSION__/$VERSION/g" "$PROJECT_DIR/packaging/Info.plist" > "$APP_DIR/Contents/Info.plist"

# Copy icon
ICON_PATH="$PROJECT_DIR/assets/icon/termpop.icns"
if [[ -f "$ICON_PATH" ]]; then
  cp "$ICON_PATH" "$APP_DIR/Contents/Resources/termpop.icns"
else
  echo "Warning: icon not found at $ICON_PATH"
fi

# --- 2. Code sign ---
if [[ -n "$SIGN_IDENTITY" ]]; then
  echo "==> Code signing with: $SIGN_IDENTITY"
  codesign --force --options runtime --deep \
    --sign "$SIGN_IDENTITY" \
    --timestamp \
    "$APP_DIR"
  echo "==> Verifying signature"
  codesign -dv --verbose=2 "$APP_DIR"
else
  echo "==> Skipping code signing (no --sign provided)"
  echo "    For local testing, ad-hoc signing..."
  codesign --force --deep --sign - "$APP_DIR"
fi

# --- 3. Create DMG ---
echo "==> Creating DMG"

DMG_TEMP="$BUILD_DIR/dmg-temp"
mkdir -p "$DMG_TEMP"
cp -R "$APP_DIR" "$DMG_TEMP/"

ln -s /Applications "$DMG_TEMP/Applications"

cp "$PROJECT_DIR/packaging/Install.command" "$DMG_TEMP/Install.command"
chmod +x "$DMG_TEMP/Install.command"

cp "$PROJECT_DIR/packaging/설치 가이드.txt" "$DMG_TEMP/설치 가이드.txt"

# Create DMG
rm -f "$DMG_PATH"
hdiutil create \
  -volname "TermPop" \
  -srcfolder "$DMG_TEMP" \
  -ov \
  -format UDZO \
  -imagekey zlib-level=9 \
  "$DMG_PATH"

# Sign DMG itself
if [[ -n "$SIGN_IDENTITY" ]]; then
  echo "==> Signing DMG"
  codesign --force --sign "$SIGN_IDENTITY" --timestamp "$DMG_PATH"
fi

# --- 4. Notarize ---
if [[ "$NOTARIZE" == true ]] && [[ -n "$SIGN_IDENTITY" ]]; then
  echo "==> Submitting for notarization"
  xcrun notarytool submit "$DMG_PATH" \
    --keychain-profile "termpop-notary" \
    --wait

  echo "==> Stapling notarization ticket"
  xcrun stapler staple "$DMG_PATH"
fi

# --- Done ---
echo ""
echo "==> Done!"
echo "    App: $APP_DIR"
echo "    DMG: $DMG_PATH"
echo "    Size: $(du -h "$DMG_PATH" | cut -f1)"
