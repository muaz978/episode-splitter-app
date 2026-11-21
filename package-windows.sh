#!/usr/bin/env bash
# Cross-compiles the Windows (64-bit) app from a Mac and zips it into dist/.
# One-time setup:  brew install mingw-w64  &&  rustup target add x86_64-pc-windows-gnu
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
cd "$(dirname "$0")"

cargo build --release --target x86_64-pc-windows-gnu

OUT="dist/Episode Part Splitter (Windows)"
rm -rf "$OUT" dist/Episode-Part-Splitter-windows.zip
mkdir -p "$OUT"
cp target/x86_64-pc-windows-gnu/release/episode-splitter.exe "$OUT/Episode Part Splitter.exe"
# Microsoft's WebView2 loader: it must sit next to the .exe.
cp "$(ls -d target/x86_64-pc-windows-gnu/release/build/webview2-com-sys-*/out/x64 | head -1)/WebView2Loader.dll" "$OUT/"
cp packaging/README-windows.txt "$OUT/READ ME FIRST.txt"

( cd dist && zip -qr -X Episode-Part-Splitter-windows.zip "Episode Part Splitter (Windows)" )
echo "Built: $OUT"
ls -la "$OUT"
du -sh dist/Episode-Part-Splitter-windows.zip
