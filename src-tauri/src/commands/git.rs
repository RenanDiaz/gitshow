use serde::Serialize;
use tauri_plugin_shell::ShellExt;

use crate::RepoState;

#[derive(Debug, Serialize)]
pub struct CommitRef {
    pub name: String,
    #[serde(rename = "type")]
    pub ref_type: String,
}

#[derive(Debug, Serialize)]
pub struct CommitLogEntry {
    pub hash: String,
    pub parents: Vec<String>,
    pub author: String,
    pub email: String,
    pub timestamp: i64,
    pub refs: Vec<CommitRef>,
    pub subject: String,
}

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

fn get_repo_path(state: &tauri::State<'_, RepoState>) -> Result<String, String> {
    let repo = state.0.lock().map_err(|e| e.to_string())?;
    repo.clone().ok_or_else(|| "No repository is open.".to_string())
}

#[tauri::command]
pub async fn get_commit_log(
    app: tauri::AppHandle,
    state: tauri::State<'_, RepoState>,
    skip: u32,
    limit: u32,
) -> Result<Vec<CommitLogEntry>, String> {
    let repo_path = get_repo_path(&state)?;

    let output = app
        .shell()
        .command("git")
        .args([
            "log",
            "--all",
            "--format=%H%x00%P%x00%an%x00%ae%x00%at%x00%D%x00%s",
            "--topo-order",
            &format!("-n{}", limit),
            &format!("--skip={}", skip),
        ])
        .current_dir(&repo_path)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries: Vec<CommitLogEntry> = Vec::new();

    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(7, '\0').collect();
        if parts.len() < 7 {
            continue;
        }

        let hash = parts[0].to_string();
        let parents: Vec<String> = if parts[1].is_empty() {
            Vec::new()
        } else {
            parts[1].split(' ').map(|s| s.to_string()).collect()
        };
        let author = parts[2].to_string();
        let email = parts[3].to_string();
        let timestamp = parts[4].parse::<i64>().unwrap_or(0);
        let refs = parse_refs(parts[5]);
        let subject = parts[6].to_string();

        entries.push(CommitLogEntry {
            hash,
            parents,
            author,
            email,
            timestamp,
            refs,
            subject,
        });
    }

    Ok(entries)
}

fn parse_refs(raw: &str) -> Vec<CommitRef> {
    if raw.is_empty() {
        return Vec::new();
    }

    let mut refs = Vec::new();

    for part in raw.split(", ") {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(branch) = trimmed.strip_prefix("HEAD -> ") {
            refs.push(CommitRef {
                name: "HEAD".to_string(),
                ref_type: "head".to_string(),
            });
            refs.push(CommitRef {
                name: branch.to_string(),
                ref_type: "local".to_string(),
            });
        } else if trimmed == "HEAD" {
            refs.push(CommitRef {
                name: "HEAD".to_string(),
                ref_type: "head".to_string(),
            });
        } else if let Some(tag) = trimmed.strip_prefix("tag: ") {
            refs.push(CommitRef {
                name: tag.to_string(),
                ref_type: "tag".to_string(),
            });
        } else if trimmed.contains('/') {
            refs.push(CommitRef {
                name: trimmed.to_string(),
                ref_type: "remote".to_string(),
            });
        } else {
            refs.push(CommitRef {
                name: trimmed.to_string(),
                ref_type: "local".to_string(),
            });
        }
    }

    refs
}

#[tauri::command]
pub async fn get_commit_files(
    app: tauri::AppHandle,
    state: tauri::State<'_, RepoState>,
    sha: String,
) -> Result<Vec<ChangedFile>, String> {
    let repo_path = get_repo_path(&state)?;

    let name_status_output = app
        .shell()
        .command("git")
        .args(["diff-tree", "--no-commit-id", "-r", "--name-status", "--find-renames", &sha])
        .current_dir(&repo_path)
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
        .current_dir(&repo_path)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !numstat_output.status.success() {
        return Err(String::from_utf8_lossy(&numstat_output.stderr).to_string());
    }

    let name_status_str = String::from_utf8_lossy(&name_status_output.stdout);
    let numstat_str = String::from_utf8_lossy(&numstat_output.stdout);

    let mut stats: std::collections::HashMap<String, (u32, u32)> = std::collections::HashMap::new();
    for line in numstat_str.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let ins = parts[0].parse::<u32>().unwrap_or(0);
            let del = parts[1].parse::<u32>().unwrap_or(0);
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
        let status = raw_status.chars().next().unwrap_or('M').to_string();

        let (path, old_path) = if status == "R" || status == "C" {
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
    state: tauri::State<'_, RepoState>,
    sha: String,
    file_path: String,
) -> Result<FileDiff, String> {
    let repo_path = get_repo_path(&state)?;

    let parent_output = app
        .shell()
        .command("git")
        .args(["rev-parse", &format!("{}^", sha)])
        .current_dir(&repo_path)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    let has_parent = parent_output.status.success();
    let parent_sha = if has_parent {
        String::from_utf8_lossy(&parent_output.stdout).trim().to_string()
    } else {
        String::new()
    };

    let original = if has_parent {
        let output = app
            .shell()
            .command("git")
            .args(["show", &format!("{}:{}", parent_sha, file_path)])
            .current_dir(&repo_path)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let mod_output = app
        .shell()
        .command("git")
        .args(["show", &format!("{}:{}", sha, file_path)])
        .current_dir(&repo_path)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    let modified = if mod_output.status.success() {
        String::from_utf8_lossy(&mod_output.stdout).to_string()
    } else {
        String::new()
    };

    Ok(FileDiff { original, modified })
}

#[tauri::command]
pub async fn get_working_directory_status(
    app: tauri::AppHandle,
    state: tauri::State<'_, RepoState>,
) -> Result<WorkingDirectoryStatus, String> {
    let repo_path = get_repo_path(&state)?;

    let status_output = app
        .shell()
        .command("git")
        .args(["status", "--porcelain=v2", "--branch"])
        .current_dir(&repo_path)
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
            let parts: Vec<&str> = line.splitn(10, ' ').collect();
            if parts.len() < 10 {
                continue;
            }
            let xy = parts[1];
            let paths_str = parts[9];
            let x = xy.chars().next().unwrap_or('.');
            let y = xy.chars().nth(1).unwrap_or('.');

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
            .current_dir(&repo_path)
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
            .current_dir(&repo_path)
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
    state: tauri::State<'_, RepoState>,
    paths: Vec<String>,
) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }

    let repo_path = get_repo_path(&state)?;

    let mut args = vec!["add", "--"];
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    args.extend(path_refs);

    let output = app
        .shell()
        .command("git")
        .args(&args)
        .current_dir(&repo_path)
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
    state: tauri::State<'_, RepoState>,
    paths: Vec<String>,
) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }

    let repo_path = get_repo_path(&state)?;

    let mut args = vec!["restore", "--staged", "--"];
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    args.extend(path_refs);

    let output = app
        .shell()
        .command("git")
        .args(&args)
        .current_dir(&repo_path)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn stage_all(
    app: tauri::AppHandle,
    state: tauri::State<'_, RepoState>,
) -> Result<(), String> {
    let repo_path = get_repo_path(&state)?;

    let output = app
        .shell()
        .command("git")
        .args(["add", "-A"])
        .current_dir(&repo_path)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}

#[tauri::command]
pub async fn unstage_all(
    app: tauri::AppHandle,
    state: tauri::State<'_, RepoState>,
) -> Result<(), String> {
    let repo_path = get_repo_path(&state)?;

    let output = app
        .shell()
        .command("git")
        .args(["reset", "HEAD"])
        .current_dir(&repo_path)
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
    state: tauri::State<'_, RepoState>,
    message: String,
    amend: bool,
) -> Result<(), String> {
    if message.is_empty() {
        return Err("Commit message cannot be empty".to_string());
    }

    let repo_path = get_repo_path(&state)?;

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
        .current_dir(&repo_path)
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
    state: tauri::State<'_, RepoState>,
    file_path: String,
    staged: bool,
) -> Result<FileDiff, String> {
    let repo_path = get_repo_path(&state)?;

    let original = if staged {
        let output = app
            .shell()
            .command("git")
            .args(["show", &format!("HEAD:{}", file_path)])
            .current_dir(&repo_path)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        }
    } else {
        let output = app
            .shell()
            .command("git")
            .args(["show", &format!(":{}", file_path)])
            .current_dir(&repo_path)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        }
    };

    let modified = if staged {
        let output = app
            .shell()
            .command("git")
            .args(["show", &format!(":{}", file_path)])
            .current_dir(&repo_path)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        }
    } else {
        // Read the working tree file using an absolute path
        let abs_path = std::path::Path::new(&repo_path).join(&file_path);
        let output = app
            .shell()
            .command("cat")
            .args([abs_path.to_string_lossy().as_ref()])
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
