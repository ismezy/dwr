use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub date: String,
    pub files_changed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReport {
    pub date: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMeta {
    pub date: String,
    pub path: String,
}

fn report_dir(project_path: &str, project_name: &str, work_dir: Option<&str>) -> PathBuf {
    if let Some(wd) = work_dir {
        PathBuf::from(wd).join(project_name)
    } else {
        PathBuf::from(project_path).join(".dwr").join("reports")
    }
}

fn report_path(project_path: &str, project_name: &str, date: &str, work_dir: Option<&str>) -> PathBuf {
    report_dir(project_path, project_name, work_dir).join(format!("{}.md", date))
}

fn run_git_log(
    project_path: &str,
    git_user_name: Option<&str>,
    date: &str,
) -> Result<Vec<CommitInfo>, String> {
    let since = format!("{} 00:00:00", date);
    let until = format!("{} 23:59:59", date);

    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(project_path)
        .arg("log")
        .arg(format!("--since={}", since))
        .arg(format!("--until={}", until))
        .arg("--pretty=format:%H|%s|%ad")
        .arg("--date=iso")
        .arg("--no-merges");

    if let Some(user) = git_user_name {
        if !user.is_empty() {
            cmd.arg(format!("--author={}", user));
        }
    }

    let output = cmd.output().map_err(|e| format!("failed to run git log: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git log failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() == 3 {
            commits.push(CommitInfo {
                hash: parts[0].to_string(),
                message: parts[1].to_string(),
                date: parts[2].to_string(),
                files_changed: Vec::new(),
            });
        }
    }

    // 获取每个 commit 的变更文件
    for commit in &mut commits {
        let files_output = Command::new("git")
            .arg("-C")
            .arg(project_path)
            .arg("diff-tree")
            .arg("--no-commit-id")
            .arg("--name-only")
            .arg("-r")
            .arg(&commit.hash)
            .output()
            .map_err(|e| format!("failed to run git diff-tree: {}", e))?;

        if files_output.status.success() {
            let files_stdout = String::from_utf8_lossy(&files_output.stdout);
            commit.files_changed = files_stdout
                .lines()
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
        }
    }

    Ok(commits)
}

fn format_report(date: &str, project_name: &str, commits: &[CommitInfo]) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {} 日报 - {}", date, project_name));
    lines.push(String::new());

    if commits.is_empty() {
        lines.push("今日无提交记录。".to_string());
    } else {
        lines.push(format!("## 提交记录 ({}条)", commits.len()));
        lines.push(String::new());

        for (i, commit) in commits.iter().enumerate() {
            lines.push(format!("{}. {}", i + 1, commit.message));
            if !commit.files_changed.is_empty() {
                lines.push(format!("   - 变更文件: {}", commit.files_changed.join(", ")));
            }
        }
    }

    lines.join("\n")
}

#[tauri::command]
pub fn generate_daily_report(
    project_path: String,
    project_name: String,
    git_user_name: Option<String>,
    date: String,
    work_dir: Option<String>,
) -> Result<DailyReport, String> {
    let commits = run_git_log(&project_path, git_user_name.as_deref(), &date)?;
    let content = format_report(&date, &project_name, &commits);

    let dir = report_dir(&project_path, &project_name, work_dir.as_deref());
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create report dir: {}", e))?;

    let path = report_path(&project_path, &project_name, &date, work_dir.as_deref());
    std::fs::write(&path, &content).map_err(|e| format!("failed to write report: {}", e))?;

    Ok(DailyReport { date, content })
}

#[tauri::command]
pub fn get_report_list(
    project_path: String,
    project_name: String,
    work_dir: Option<String>,
) -> Result<Vec<ReportMeta>, String> {
    let dir = report_dir(&project_path, &project_name, work_dir.as_deref());
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let date = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            entries.push(ReportMeta {
                date: date.clone(),
                path: path.to_string_lossy().to_string(),
            });
        }
    }

    entries.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(entries)
}

#[tauri::command]
pub fn read_report(
    project_path: String,
    project_name: String,
    date: String,
    work_dir: Option<String>,
) -> Result<String, String> {
    let path = report_path(&project_path, &project_name, &date, work_dir.as_deref());
    if !path.exists() {
        return Err("report not found".to_string());
    }
    std::fs::read_to_string(&path).map_err(|e| format!("failed to read report: {}", e))
}
