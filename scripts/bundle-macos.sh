#!/usr/bin/env bash
# Builds a universal (Apple Silicon + Intel) MetroMind.app and zips it.
#
# Runs on macOS with Rust and the Xcode Command Line Tools installed. No Apple
# developer account is needed: the app is ad-hoc signed, not notarized, so on
# first launch macOS asks you to allow it (see README, "Mac").
#
# Usage: scripts/bundle-macos.sh [output-dir]   (default: dist/macos)
set -euo pipefail

cd "$(dirname "$0")/.."

OUT_DIR=${1:-dist/macos}
APP="$OUT_DIR/MetroMind.app"
ZIP="$OUT_DIR/MetroMind-macos.zip"
ICON_SRC=crates/metromind-app/assets/icon.png
TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)

echo "==> Building release binaries"
rustup target add "${TARGETS[@]}"
for target in "${TARGETS[@]}"; do
    cargo build --release --locked -p metromind-app --target "$target"
done

echo "==> Assembling $APP"
rm -rf "$APP" "$ZIP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/MetroMind" \
    target/aarch64-apple-darwin/release/metromind-app \
    target/x86_64-apple-darwin/release/metromind-app

# The icon source is 200 px, so the iconset stops at 256 px to limit upscaling.
iconset="$(mktemp -d)/MetroMind.iconset"
mkdir -p "$iconset"
resize() { sips -z "$1" "$1" "$ICON_SRC" --out "$iconset/$2" >/dev/null; }
resize 16 icon_16x16.png
resize 32 icon_16x16@2x.png
resize 32 icon_32x32.png
resize 64 icon_32x32@2x.png
resize 128 icon_128x128.png
resize 256 icon_128x128@2x.png
resize 256 icon_256x256.png
iconutil -c icns "$iconset" -o "$APP/Contents/Resources/MetroMind.icns"

version=$(cargo pkgid -p metromind-app | sed 's/.*[#@]//')
cat >"$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>                <string>MetroMind</string>
    <key>CFBundleDisplayName</key>         <string>MetroMind</string>
    <key>CFBundleIdentifier</key>          <string>app.metromind.MetroMind</string>
    <key>CFBundleExecutable</key>          <string>MetroMind</string>
    <key>CFBundleIconFile</key>            <string>MetroMind</string>
    <key>CFBundlePackageType</key>         <string>APPL</string>
    <key>CFBundleShortVersionString</key>  <string>$version</string>
    <key>CFBundleVersion</key>             <string>$version</string>
    <key>LSMinimumSystemVersion</key>      <string>11.0</string>
    <key>NSHighResolutionCapable</key>     <true/>
</dict>
</plist>
EOF
plutil -lint "$APP/Contents/Info.plist"

echo "==> Ad-hoc signing"
codesign --force --sign - "$APP"
codesign --verify --strict "$APP"

echo "==> Zipping"
ditto -c -k --keepParent "$APP" "$ZIP"
echo "Done: $ZIP"
