fn main() {
    // Where mpv.lib / libmpv.dll.a lives. build-windows.ps1 sets MPV_DIR; default: deps/mpv
    let dir = std::env::var("MPV_DIR").unwrap_or_else(|_| {
        format!("{}/deps/mpv", std::env::var("CARGO_MANIFEST_DIR").unwrap())
    });
    println!("cargo:rustc-link-search=native={dir}");
    println!("cargo:rerun-if-env-changed=MPV_DIR");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "RustPlayer");
        res.set("FileDescription", "RustPlayer media player");
        if let Err(e) = res.compile() {
            println!("cargo:warning=icon not embedded: {e}");
        }
    }
}
