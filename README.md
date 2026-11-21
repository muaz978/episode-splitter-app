# Episode Part Splitter (desktop app)

A tiny native wrapper around the offline HTML tool in `assets/index.html`. The page is embedded in
the binary and shown in the system web view (WebKit on macOS, WebView2 on Windows), so the UI and
logic are exactly the web version. The Rust code adds only three things:

- a macOS menu bar (so Cmd+C / Cmd+V / Cmd+Z work in the text fields),
- settings saved to a file in the user's app-data folder instead of browser storage (`src/shim.js`),
- a native clipboard so "Copy for email" puts real HTML on the clipboard (`arboard`).

## Update the UI
Replace `assets/index.html` with the new version of the page, then rebuild.

## Build
```
./package-mac.sh        # universal Mac app  -> dist/Episode-Part-Splitter-mac.zip
./package-windows.sh    # Windows 64-bit     -> dist/Episode-Part-Splitter-windows.zip
```
One-time setup on a Mac: Rust (`rustup`), `brew install mingw-w64`, `rustup target add x86_64-pc-windows-gnu`.
The Windows build made on a Mac ships an `.exe` plus `WebView2Loader.dll`. For a single-file `.exe`
use the GitHub Actions workflow in `.github/workflows/build.yml` (MSVC toolchain).

Icons are generated from a script (Pillow); the results are committed in `assets/`.
