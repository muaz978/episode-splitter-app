#!/usr/bin/env bash
# Builds "Episode Part Splitter.app" as one universal app (Apple Silicon + Intel)
# and zips it into dist/. Run on a Mac with Rust installed.
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
cd "$(dirname "$0")"

rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

STAGE="dist/Episode Part Splitter (Mac)"
APP="$STAGE/Episode Part Splitter.app"
rm -rf "dist/Episode Part Splitter.app" "$STAGE" dist/Episode-Part-Splitter-mac.zip
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

lipo -create \
  target/aarch64-apple-darwin/release/episode-splitter \
  target/x86_64-apple-darwin/release/episode-splitter \
  -output "$APP/Contents/MacOS/EpisodeSplitter"
cp assets/icon.icns "$APP/Contents/Resources/icon.icns"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleName</key><string>Episode Part Splitter</string>
  <key>CFBundleDisplayName</key><string>Episode Part Splitter</string>
  <key>CFBundleExecutable</key><string>EpisodeSplitter</string>
  <key>CFBundleIdentifier</key><string>com.hilalplay.episodesplitter</string>
  <key>CFBundleIconFile</key><string>icon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>1.0.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>14.5</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.productivity</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict>
</plist>
PLIST

# Ad-hoc signature: enough to run on this Mac and to keep the bundle intact.
# (Other Macs will ask for a one-time "Open anyway" because it is not notarized.)
codesign --force --deep --sign - "$APP"
codesign --verify --deep --strict "$APP"

cp packaging/README-mac.txt "$STAGE/READ ME FIRST.txt"
ditto -c -k --norsrc --noextattr --noqtn --keepParent "$STAGE" dist/Episode-Part-Splitter-mac.zip
echo "Built: $APP"
lipo -archs "$APP/Contents/MacOS/EpisodeSplitter"
du -sh "$APP" dist/Episode-Part-Splitter-mac.zip
