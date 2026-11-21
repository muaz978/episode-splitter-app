#!/usr/bin/env bash
# Cross-compiles the Windows (64-bit) app from a Mac as ONE standalone .exe and zips it into dist/.
# One-time setup:
#   brew install llvm lld && cargo install cargo-xwin --locked && rustup target add x86_64-pc-windows-msvc
# (cargo-xwin downloads Microsoft's Windows build libraries; using them means accepting Microsoft's license terms.)
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:/opt/homebrew/opt/llvm/bin:/opt/homebrew/opt/lld/bin:$PATH"
cd "$(dirname "$0")"

cargo xwin build --release --target x86_64-pc-windows-msvc

OUT="dist/Episode Part Splitter (Windows)"
rm -rf "$OUT" dist/Episode-Part-Splitter-windows.zip
mkdir -p "$OUT"
cp target/x86_64-pc-windows-msvc/release/episode-splitter.exe "$OUT/Episode Part Splitter.exe"
cp packaging/README-windows.txt "$OUT/READ ME FIRST.txt"

( cd dist && zip -qr -X Episode-Part-Splitter-windows.zip "Episode Part Splitter (Windows)" )
echo "Built: $OUT"
ls -la "$OUT"
du -sh dist/Episode-Part-Splitter-windows.zip
