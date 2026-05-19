mod ai;
mod config;
mod project;
mod report;
mod summary;

use tauri::Manager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db = project::init_db(app.handle())?;
            app.manage(db);
            let config_db = config::init_config_db(app.handle())?;
            app.manage(config_db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            project::create_project,
            project::get_projects,
            project::update_project,
            project::delete_project,
            config::get_configs,
            config::save_configs,
            report::generate_daily_report,
            report::get_report_list,
            report::read_report,
            report::save_report,
            report::polish_report,
            summary::generate_summary_report,
            summary::get_summary_report_list,
            summary::read_summary_report,
            summary::save_summary_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
