use std::process::Command;
use tauri::AppHandle;

#[derive(Clone, serde::Serialize, specta::Type, Debug)]
pub struct GitBranchInfo {
    pub branch: Option<String>,
}

/// Get the current Git branch for a given directory
#[tauri::command]
#[specta::specta]
pub fn get_git_branch(_app: AppHandle, directory: String) -> Result<GitBranchInfo, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&directory)
        .output()
        .map_err(|e| format!("Failed to execute git command: {e}"))?;

    if !output.status.success() {
        // Not a git repository or git not available
        return Ok(GitBranchInfo { branch: None });
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if branch.is_empty() {
        return Ok(GitBranchInfo { branch: None });
    }

    Ok(GitBranchInfo {
        branch: Some(branch),
    })
}
