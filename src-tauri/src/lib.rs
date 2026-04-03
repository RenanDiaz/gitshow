mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::git::get_commit_files,
            commands::git::get_file_diff,
            commands::git::get_working_directory_status,
            commands::git::stage_files,
            commands::git::unstage_files,
            commands::git::stage_all,
            commands::git::unstage_all,
            commands::git::create_commit,
            commands::git::get_working_directory_diff,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
