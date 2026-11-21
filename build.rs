// Embeds the app icon and version info into the Windows .exe (no effect on macOS).
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Episode Part Splitter");
        res.set("FileDescription", "Episode Part Splitter");
        res.set("CompanyName", "HilalPlay");
        res.set("LegalCopyright", "HilalPlay");
        if let Err(e) = res.compile() {
            // A missing resource compiler must not block the build; the app just gets no Explorer icon.
            println!("cargo:warning=could not embed the Windows icon: {e}");
        }
    }
}
