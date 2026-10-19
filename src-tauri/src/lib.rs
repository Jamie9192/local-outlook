mod model;
mod providers;
mod service;
mod store;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            app.manage(service::AppState::new(&dir)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            service::bootstrap,
            service::check_area,
            service::save_settings,
            service::clear_cache,
            service::test_provider
        ])
        .run(tauri::generate_context!())
        .expect("Unable to start Local Outlook");
}
