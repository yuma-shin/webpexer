pub mod commands;
pub mod converter;
pub mod encoding;
pub mod errors;
pub mod history;
pub mod models;
pub mod output;
pub mod settings;
pub mod validator;

#[cfg(test)]
mod validator_tests;

#[cfg(test)]
mod output_tests;

#[cfg(test)]
mod commands_tests;

#[cfg(test)]
mod history_tests;

use tauri::Manager;

use commands::ConversionState;
use history::HistoryState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(ConversionState::new())
        .setup(|app| {
            let _window = app.get_webview_window("main").unwrap();

            // 履歴ストアの初期化（app_data_dir に history.json を永続化）
            let app_data_dir = app.path().app_data_dir().map_err(|e| {
                Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                    as Box<dyn std::error::Error>
            })?;
            let history_state = HistoryState::new(&app_data_dir);
            app.manage(history_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_conversion,
            commands::cancel_conversion,
            commands::select_folder,
            commands::get_supported_formats,
            commands::validate_output_path,
            commands::check_disk_space,
            commands::check_large_files,
            commands::get_conversion_history,
            commands::clear_conversion_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
