mod commands;

use std::sync::Mutex;

pub struct RepoState(pub Mutex<Option<String>>);

#[tauri::command]
async fn open_repository(
    app: tauri::AppHandle,
    path: String,
    state: tauri::State<'_, RepoState>,
) -> Result<String, String> {
    use tauri_plugin_shell::ShellExt;

    // Verify it's a git repo
    let output = app
        .shell()
        .command("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(&path)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("The selected folder is not a git repository.".to_string());
    }

    let mut repo = state.0.lock().map_err(|e| e.to_string())?;
    *repo = Some(path.clone());
    Ok(path)
}

#[tauri::command]
fn get_current_repo(state: tauri::State<'_, RepoState>) -> Result<Option<String>, String> {
    let repo = state.0.lock().map_err(|e| e.to_string())?;
    Ok(repo.clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(RepoState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            open_repository,
            get_current_repo,
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
