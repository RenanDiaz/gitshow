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

#[derive(Debug, Serialize)]
pub struct WorkingDirectoryStatus {
    pub staged: Vec<ChangedFile>,
    pub unstaged: Vec<ChangedFile>,
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

#[tauri::command]
pub async fn get_working_directory_status(
    app: tauri::AppHandle,
) -> Result<WorkingDirectoryStatus, String> {
    let status_output = app
        .shell()
        .command("git")
        .args(["status", "--porcelain=v2", "--branch"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !status_output.status.success() {
        return Err(String::from_utf8_lossy(&status_output.stderr).to_string());
    }

    let status_str = String::from_utf8_lossy(&status_output.stdout);

    let mut staged: Vec<ChangedFile> = Vec::new();
    let mut unstaged: Vec<ChangedFile> = Vec::new();

    for line in status_str.lines() {
        if line.starts_with('#') {
            continue;
        }

        if line.starts_with('1') {
            // Ordinary changed entry: 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
            let parts: Vec<&str> = line.splitn(9, ' ').collect();
            if parts.len() < 9 {
                continue;
            }
            let xy = parts[1];
            let path = parts[8].to_string();
            let x = xy.chars().next().unwrap_or('.');
            let y = xy.chars().nth(1).unwrap_or('.');

            if x != '.' {
                let status = match x {
                    'A' => "A",
                    'D' => "D",
                    'M' => "M",
                    _ => "M",
                };
                staged.push(ChangedFile {
                    status: status.to_string(),
                    path: path.clone(),
                    old_path: None,
                    insertions: 0,
                    deletions: 0,
                });
            }

            if y != '.' {
                let status = match y {
                    'A' => "A",
                    'D' => "D",
                    'M' => "M",
                    _ => "M",
                };
                unstaged.push(ChangedFile {
                    status: status.to_string(),
                    path,
                    old_path: None,
                    insertions: 0,
                    deletions: 0,
                });
            }
        } else if line.starts_with('2') {
            // Renamed/copied entry: 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
            let parts: Vec<&str> = line.splitn(10, ' ').collect();
            if parts.len() < 10 {
                continue;
            }
            let xy = parts[1];
            let paths_str = parts[9];
            let x = xy.chars().next().unwrap_or('.');
            let y = xy.chars().nth(1).unwrap_or('.');

            // paths are separated by tab: path\torigPath
            let path_parts: Vec<&str> = paths_str.split('\t').collect();
            let path = path_parts[0].to_string();
            let orig_path = path_parts.get(1).map(|s| s.to_string());

            if x != '.' {
                let status = match x {
                    'R' => "R",
                    'C' => "C",
                    _ => "M",
                };
                staged.push(ChangedFile {
                    status: status.to_string(),
                    path: path.clone(),
                    old_path: orig_path.clone(),
                    insertions: 0,
                    deletions: 0,
                });
            }

            if y != '.' {
                unstaged.push(ChangedFile {
                    status: "M".to_string(),
                    path,
                    old_path: None,
                    insertions: 0,
                    deletions: 0,
                });
            }
        } else if line.starts_with('?') {
            // Untracked: ? <path>
            let path = line[2..].to_string();
            unstaged.push(ChangedFile {
                status: "A".to_string(),
                path,
                old_path: None,
                insertions: 0,
                deletions: 0,
            });
        }
    }

    // Get numstat for staged files
    if !staged.is_empty() {
        let diff_output = app
            .shell()
            .command("git")
            .args(["diff", "--cached", "--numstat"])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if diff_output.status.success() {
            let diff_str = String::from_utf8_lossy(&diff_output.stdout);
            let mut stats: std::collections::HashMap<String, (u32, u32)> =
                std::collections::HashMap::new();
            for stat_line in diff_str.lines() {
                let stat_parts: Vec<&str> = stat_line.split('\t').collect();
                if stat_parts.len() >= 3 {
                    let ins = stat_parts[0].parse::<u32>().unwrap_or(0);
                    let del = stat_parts[1].parse::<u32>().unwrap_or(0);
                    let p = stat_parts.last().unwrap().to_string();
                    stats.insert(p, (ins, del));
                }
            }
            for file in &mut staged {
                if let Some(&(ins, del)) = stats.get(&file.path) {
                    file.insertions = ins;
                    file.deletions = del;
                }
            }
        }
    }

    // Get numstat for unstaged files
    if !unstaged.is_empty() {
        let diff_output = app
            .shell()
            .command("git")
            .args(["diff", "--numstat"])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if diff_output.status.success() {
            let diff_str = String::from_utf8_lossy(&diff_output.stdout);
            let mut stats: std::collections::HashMap<String, (u32, u32)> =
                std::collections::HashMap::new();
            for stat_line in diff_str.lines() {
                let stat_parts: Vec<&str> = stat_line.split('\t').collect();
                if stat_parts.len() >= 3 {
                    let ins = stat_parts[0].parse::<u32>().unwrap_or(0);
                    let del = stat_parts[1].parse::<u32>().unwrap_or(0);
                    let p = stat_parts.last().unwrap().to_string();
                    stats.insert(p, (ins, del));
                }
            }
            for file in &mut unstaged {
                if let Some(&(ins, del)) = stats.get(&file.path) {
                    file.insertions = ins;
                    file.deletions = del;
                }
            }
        }
    }

    Ok(WorkingDirectoryStatus { staged, unstaged })
}

#[tauri::command]
pub async fn stage_files(
    app: tauri::AppHandle,
    paths: Vec<String>,
) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }

    let mut args = vec!["add", "--"];
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    args.extend(path_refs);

    let output = app
        .shell()
        .command("git")
        .args(&args)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn unstage_files(
    app: tauri::AppHandle,
    paths: Vec<String>,
) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }

    let mut args = vec!["restore", "--staged", "--"];
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    args.extend(path_refs);

    let output = app
        .shell()
        .command("git")
        .args(&args)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn stage_all(app: tauri::AppHandle) -> Result<(), String> {
    let output = app
        .shell()
        .command("git")
        .args(["add", "-A"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn unstage_all(app: tauri::AppHandle) -> Result<(), String> {
    let output = app
        .shell()
        .command("git")
        .args(["reset", "HEAD"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn create_commit(
    app: tauri::AppHandle,
    message: String,
    amend: bool,
) -> Result<(), String> {
    if message.is_empty() {
        return Err("Commit message cannot be empty".to_string());
    }

    let mut args = vec!["commit"];
    if amend {
        args.push("--amend");
    }
    args.push("-m");
    args.push(&message);

    let output = app
        .shell()
        .command("git")
        .args(&args)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn get_working_directory_diff(
    app: tauri::AppHandle,
    file_path: String,
    staged: bool,
) -> Result<FileDiff, String> {
    // Get the index/HEAD version (original)
    let original = if staged {
        // For staged: original is HEAD version
        let output = app
            .shell()
            .command("git")
            .args(["show", &format!("HEAD:{}", file_path)])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        }
    } else {
        // For unstaged: original is index version (staged or HEAD)
        let output = app
            .shell()
            .command("git")
            .args(["show", &format!(":{}", file_path)])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        }
    };

    // Get the modified version
    let modified = if staged {
        // For staged: modified is the index version
        let output = app
            .shell()
            .command("git")
            .args(["show", &format!(":{}", file_path)])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        }
    } else {
        // For unstaged: modified is the working tree version — read the file directly
        let output = app
            .shell()
            .command("cat")
            .args([&file_path])
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        }
    };

    Ok(FileDiff { original, modified })
}
