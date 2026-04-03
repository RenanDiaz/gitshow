use serde::Serialize;
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Serialize)]
pub struct FileDiff {
    pub original: String,
    pub modified: String,
}

#[derive(Debug, Serialize)]
pub struct ChangedFile {
    pub status: String,
    pub path: String,
    pub old_path: Option<String>,
    pub insertions: u32,
    pub deletions: u32,
}

#[tauri::command]
pub async fn get_commit_files(
    app: tauri::AppHandle,
    sha: String,
) -> Result<Vec<ChangedFile>, String> {
    let name_status_output = app
        .shell()
        .command("git")
        .args(["diff-tree", "--no-commit-id", "-r", "--name-status", "--find-renames", &sha])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !name_status_output.status.success() {
        return Err(String::from_utf8_lossy(&name_status_output.stderr).to_string());
    }

    let numstat_output = app
        .shell()
        .command("git")
        .args(["diff-tree", "--no-commit-id", "-r", "--numstat", &sha])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !numstat_output.status.success() {
        return Err(String::from_utf8_lossy(&numstat_output.stderr).to_string());
    }

    let name_status_str = String::from_utf8_lossy(&name_status_output.stdout);
    let numstat_str = String::from_utf8_lossy(&numstat_output.stdout);

    // Parse numstat into a map: path -> (insertions, deletions)
    let mut stats: std::collections::HashMap<String, (u32, u32)> = std::collections::HashMap::new();
    for line in numstat_str.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let ins = parts[0].parse::<u32>().unwrap_or(0);
            let del = parts[1].parse::<u32>().unwrap_or(0);
            // For renames, numstat shows the new path (last element)
            let path = parts.last().unwrap().to_string();
            stats.insert(path, (ins, del));
        }
    }

    let mut files: Vec<ChangedFile> = Vec::new();

    for line in name_status_str.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
            continue;
        }

        let raw_status = parts[0];
        // Rename status looks like R085, extract just the letter
        let status = raw_status.chars().next().unwrap_or('M').to_string();

        let (path, old_path) = if status == "R" || status == "C" {
            // Rename/Copy: old_path -> new_path
            if parts.len() >= 3 {
                (parts[2].to_string(), Some(parts[1].to_string()))
            } else {
                continue;
            }
        } else if parts.len() >= 2 {
            (parts[1].to_string(), None)
        } else {
            continue;
        };

        let (insertions, deletions) = stats.get(&path).copied().unwrap_or((0, 0));

        files.push(ChangedFile {
            status,
            path,
            old_path,
            insertions,
            deletions,
        });
    }

    Ok(files)
}

#[tauri::command]
pub async fn get_file_diff(
    app: tauri::AppHandle,
    sha: String,
    file_path: String,
) -> Result<FileDiff, String> {
    // Get parent SHA
    let parent_output = app
        .shell()
        .command("git")
        .args(["rev-parse", &format!("{}^", sha)])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    let has_parent = parent_output.status.success();
    let parent_sha = if has_parent {
        String::from_utf8_lossy(&parent_output.stdout).trim().to_string()
    } else {
        String::new()
    };

    // Get original content (parent version)
    let original = if has_parent {
        let output = app
            .shell()
            .command("git")
            .args(["show", &format!("{}:{}", parent_sha, file_path)])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            // File didn't exist in parent (added file)
            String::new()
        }
    } else {
        // No parent (initial commit) — original is empty
        String::new()
    };

    // Get modified content (current commit version)
    let mod_output = app
        .shell()
        .command("git")
        .args(["show", &format!("{}:{}", sha, file_path)])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    let modified = if mod_output.status.success() {
        String::from_utf8_lossy(&mod_output.stdout).to_string()
    } else {
        // File doesn't exist at this commit (deleted file)
        String::new()
    };

    Ok(FileDiff { original, modified })
}
