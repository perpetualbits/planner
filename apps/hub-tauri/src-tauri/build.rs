// build.rs — Tauri build script.
//
// Required by Tauri 2 to generate the code-signing configuration and
// platform-specific metadata at compile time.
fn main() {
    tauri_build::build()
}
