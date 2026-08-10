include!("src/command_names.rs");

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(ALL_COMMAND_NAMES)),
    )
    .expect("failed to run tauri-build");
}
