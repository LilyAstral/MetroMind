//! Embeds the app icon in the Windows executable, so Explorer, shortcuts and
//! `.metromind` files (whose DefaultIcon points at the exe) show it.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()
            .expect("failed to embed the Windows icon resource");
    }
}
