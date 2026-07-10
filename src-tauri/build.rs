fn main() {
    // On Windows with non-ASCII paths (e.g., Japanese characters in path),
    // rc.exe cannot handle resource file paths with non-ASCII characters.
    // Workaround: copy icon to an ASCII-safe temp path and point winres there.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();

    if cfg!(target_os = "windows") && !manifest_dir.is_ascii() {
        let temp_icon = std::path::PathBuf::from(r"C:\tmp\tauri-icons\icon.ico");
        if temp_icon.exists() {
            let win_attrs =
                tauri_build::WindowsAttributes::new().window_icon_path(&temp_icon);
            let attributes = tauri_build::Attributes::new().windows_attributes(win_attrs);
            tauri_build::try_build(attributes).expect("failed to run tauri build");
        } else {
            // If no icon available at temp path, panic with helpful message
            panic!(
                "Icon file not found at {}. \
                 The project is in a non-ASCII path which causes rc.exe issues. \
                 Please run create-ico.ps1 first or move the project to an ASCII-only path.",
                temp_icon.display()
            );
        }
    } else {
        tauri_build::build()
    }
}
