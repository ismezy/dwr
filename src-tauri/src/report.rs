use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

use crate::ai;
use crate::config::ConfigDb;

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
    let parts: Vec<&str> = date.split('-').collect();
    let year = parts.get(0).unwrap_or(&"");
    let month = parts.get(1).unwrap_or(&"");
    report_dir(project_path, project_name, work_dir).join(year).join(month).join(format!("{}.md", date))
}

fn weekly_report_dir(project_path: &str, project_name: &str, work_dir: Option<&str>) -> PathBuf {
    report_dir(project_path, project_name, work_dir).join("weekly")
}

fn weekly_report_path(project_path: &str, project_name: &str, week_start: &str, week_end: &str, work_dir: Option<&str>) -> PathBuf {
    let year = week_start.split('-').next().unwrap_or("");
    weekly_report_dir(project_path, project_name, work_dir).join(year).join(format!("{}至{}.md", week_start, week_end))
}

pub fn run_git_log(
    project_path: &str,
    git_user_name: Option<&str>,
    since: &str,
    until: &str,
) -> Result<Vec<CommitInfo>, String> {
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

pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30,
    }
}

pub fn next_date(year: i32, month: u32, day: u32) -> (i32, u32, u32) {
    let dim = days_in_month(year, month);
    if day < dim {
        (year, month, day + 1)
    } else if month < 12 {
        (year, month + 1, 1)
    } else {
        (year + 1, 1, 1)
    }
}

pub fn parse_date(date: &str) -> Result<(i32, u32, u32), String> {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(format!("invalid date: {}", date));
    }
    let year = parts[0].parse().map_err(|_| format!("invalid year"))?;
    let month = parts[1].parse().map_err(|_| format!("invalid month"))?;
    let day = parts[2].parse().map_err(|_| format!("invalid day"))?;
    Ok((year, month, day))
}

pub fn format_date(year: i32, month: u32, day: u32) -> String {
    format!("{:04}-{:02}-{:02}", year, month, day)
}

pub fn collect_weekly_daily_reports(
    project_path: &str,
    project_name: &str,
    week_start: &str,
    week_end: &str,
    work_dir: Option<&str>,
) -> Vec<(String, String)> {
    let mut reports = Vec::new();
    let Ok((mut y, mut m, mut d)) = parse_date(week_start) else { return reports; };
    let Ok((end_y, end_m, end_d)) = parse_date(week_end) else { return reports; };

    loop {
        let date_str = format_date(y, m, d);
        let path = report_path(project_path, project_name, &date_str, work_dir);
        if let Ok(content) = std::fs::read_to_string(&path) {
            reports.push((date_str, content));
        }

        if y == end_y && m == end_m && d == end_d {
            break;
        }
        (y, m, d) = next_date(y, m, d);
    }

    reports
}

fn format_weekly_report(week_start: &str, week_end: &str, project_name: &str, daily_reports: &[(String, String)]) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {} ~ {} 周报 - {}", week_start, week_end, project_name));
    lines.push(String::new());

    if daily_reports.is_empty() {
        lines.push("本周暂无日报内容。".to_string());
    } else {
        for (date, content) in daily_reports {
            lines.push(format!("## {}", date));
            lines.push(content.clone());
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

fn build_commits_text(commits: &[CommitInfo]) -> String {
    if commits.is_empty() {
        return "今日无提交记录。".to_string();
    }
    let mut text = String::new();
    for (i, commit) in commits.iter().enumerate() {
        text.push_str(&format!("{}. {}\n", i + 1, commit.message));
        if !commit.files_changed.is_empty() {
            text.push_str(&format!("   变更文件: {}\n", commit.files_changed.join(", ")));
        }
    }
    text
}

fn build_ai_prompt(date: &str, project_name: &str, commits: &[CommitInfo], template: Option<&str>) -> String {
    let commits_text = build_commits_text(commits);

    let mut prompt = format!(
        "请根据以下 Git 提交记录生成一份工作日报。\n\n项目：{}\n日期：{}\n\n提交记录：\n{}",
        project_name, date, commits_text
    );

    if let Some(tpl) = template {
        prompt.push('\n');
        prompt.push_str(tpl.trim());
    } else {
        prompt.push_str("\n要求：\n");
        prompt.push_str("- 用第一人称描述今天的工作内容\n");
        prompt.push_str("- 按工作内容分类汇总\n");
        prompt.push_str("- 语言简洁专业\n");
        prompt.push_str("- 输出 Markdown 格式，只输出日报正文，不需要标题以外的额外说明\n");
    }

    prompt
}

fn build_weekly_ai_prompt(week_start: &str, week_end: &str, project_name: &str, daily_reports: &[(String, String)], template: Option<&str>) -> String {
    let mut daily_text = String::new();
    for (date, content) in daily_reports {
        daily_text.push_str(&format!("\n## {}\n{}", date, content));
    }

    let mut prompt = format!(
        "请根据以下本周各日的日报内容，汇总生成一份工作周报。\n\n项目：{}\n周期：{} ~ {}\n\n日报内容：{}",
        project_name, week_start, week_end, daily_text
    );

    if let Some(tpl) = template {
        prompt.push('\n');
        prompt.push_str(tpl.trim());
    } else {
        prompt.push_str("\n要求：\n");
        prompt.push_str("- 用第一人称描述本周的工作内容\n");
        prompt.push_str("- 基于已有的日报内容，进行概括和汇总，不要遗漏重要工作\n");
        prompt.push_str("- 语言简洁专业\n");
        prompt.push_str("- 输出 Markdown 格式，只输出周报正文，不需要标题以外的额外说明\n");
    }

    prompt
}

#[tauri::command]
pub async fn generate_daily_report(
    state: tauri::State<'_, ConfigDb>,
    project_path: String,
    project_name: String,
    git_user_name: Option<String>,
    date: String,
    work_dir: Option<String>,
) -> Result<DailyReport, String> {
    let since = format!("{} 00:00:00", date);
    let until = format!("{} 23:59:59", date);
    let commits = run_git_log(&project_path, git_user_name.as_deref(), &since, &until)?;

    let configs = crate::config::get_configs(state)?;

    let content = if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        if commits.is_empty() {
            format_report(&date, &project_name, &commits)
        } else {
            let prompt = build_ai_prompt(&date, &project_name, &commits, configs.ai_template.as_deref());
            let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
            client.generate(&prompt)?
        }
    } else {
        format_report(&date, &project_name, &commits)
    };

    let dir = report_dir(&project_path, &project_name, work_dir.as_deref());
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create report dir: {}", e))?;

    let path = report_path(&project_path, &project_name, &date, work_dir.as_deref());
    std::fs::write(&path, &content).map_err(|e| format!("failed to write report: {}", e))?;

    Ok(DailyReport { date, content })
}

#[tauri::command]
pub async fn generate_weekly_report(
    state: tauri::State<'_, ConfigDb>,
    project_path: String,
    project_name: String,
    _git_user_name: Option<String>,
    week_start: String,
    week_end: String,
    work_dir: Option<String>,
) -> Result<DailyReport, String> {
    let daily_reports = collect_weekly_daily_reports(&project_path, &project_name, &week_start, &week_end, work_dir.as_deref());

    let configs = crate::config::get_configs(state)?;

    let content = if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        if daily_reports.is_empty() {
            format_weekly_report(&week_start, &week_end, &project_name, &daily_reports)
        } else {
            let prompt = build_weekly_ai_prompt(&week_start, &week_end, &project_name, &daily_reports, configs.ai_template.as_deref());
            let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
            client.generate(&prompt)?
        }
    } else {
        format_weekly_report(&week_start, &week_end, &project_name, &daily_reports)
    };

    let dir = weekly_report_dir(&project_path, &project_name, work_dir.as_deref()).join(week_start.split('-').next().unwrap_or(""));
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create weekly report dir: {}", e))?;

    let path = weekly_report_path(&project_path, &project_name, &week_start, &week_end, work_dir.as_deref());
    std::fs::write(&path, &content).map_err(|e| format!("failed to write weekly report: {}", e))?;

    Ok(DailyReport { date: week_start, content })
}

#[tauri::command]
pub fn get_report_list(
    project_path: String,
    project_name: String,
    work_dir: Option<String>,
) -> Result<Vec<ReportMeta>, String> {
    let base_dir = report_dir(&project_path, &project_name, work_dir.as_deref());
    if !base_dir.exists() {
        std::fs::create_dir_all(&base_dir).map_err(|e| format!("failed to create report dir: {}", e))?;
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for year_entry in std::fs::read_dir(&base_dir).map_err(|e| format!("failed to read dir: {}", e))? {
        let year_entry = year_entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let year_path = year_entry.path();
        if !year_path.is_dir() { continue; }
        for month_entry in std::fs::read_dir(&year_path).map_err(|e| format!("failed to read dir: {}", e))? {
            let month_entry = month_entry.map_err(|e| format!("failed to read entry: {}", e))?;
            let month_path = month_entry.path();
            if !month_path.is_dir() { continue; }
            for file_entry in std::fs::read_dir(&month_path).map_err(|e| format!("failed to read dir: {}", e))? {
                let file_entry = file_entry.map_err(|e| format!("failed to read entry: {}", e))?;
                let path = file_entry.path();
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

#[tauri::command]
pub fn save_report(
    project_path: String,
    project_name: String,
    date: String,
    content: String,
    work_dir: Option<String>,
) -> Result<(), String> {
    let path = report_path(&project_path, &project_name, &date, work_dir.as_deref());
    std::fs::write(&path, content).map_err(|e| format!("failed to save report: {}", e))
}

#[tauri::command]
pub fn get_weekly_report_list(
    project_path: String,
    project_name: String,
    work_dir: Option<String>,
) -> Result<Vec<ReportMeta>, String> {
    let base_dir = weekly_report_dir(&project_path, &project_name, work_dir.as_deref());
    if !base_dir.exists() {
        std::fs::create_dir_all(&base_dir).map_err(|e| format!("failed to create weekly report dir: {}", e))?;
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for year_entry in std::fs::read_dir(&base_dir).map_err(|e| format!("failed to read dir: {}", e))? {
        let year_entry = year_entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let year_path = year_entry.path();
        if !year_path.is_dir() { continue; }
        for file_entry in std::fs::read_dir(&year_path).map_err(|e| format!("failed to read dir: {}", e))? {
            let file_entry = file_entry.map_err(|e| format!("failed to read entry: {}", e))?;
            let path = file_entry.path();
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
    }

    entries.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(entries)
}

#[tauri::command]
pub fn read_weekly_report(
    project_path: String,
    project_name: String,
    week_start: String,
    work_dir: Option<String>,
) -> Result<String, String> {
    let year = week_start.split('-').next().unwrap_or("");
    let dir = weekly_report_dir(&project_path, &project_name, work_dir.as_deref()).join(year);
    if !dir.exists() {
        return Err("report not found".to_string());
    }
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if stem.starts_with(&week_start) {
                return std::fs::read_to_string(&path).map_err(|e| format!("failed to read weekly report: {}", e));
            }
        }
    }
    Err("report not found".to_string())
}

#[tauri::command]
pub fn save_weekly_report(
    project_path: String,
    project_name: String,
    week_start: String,
    content: String,
    work_dir: Option<String>,
) -> Result<(), String> {
    let year = week_start.split('-').next().unwrap_or("");
    let dir = weekly_report_dir(&project_path, &project_name, work_dir.as_deref()).join(year);
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create weekly report dir: {}", e))?;

    let mut target_path: Option<PathBuf> = None;
    if dir.exists() {
        for entry in std::fs::read_dir(&dir).map_err(|e| format!("failed to read dir: {}", e))? {
            let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if stem.starts_with(&week_start) {
                    target_path = Some(path);
                    break;
                }
            }
        }
    }

    if let Some(path) = target_path {
        std::fs::write(&path, content).map_err(|e| format!("failed to save weekly report: {}", e))
    } else {
        Err("report not found".to_string())
    }
}

#[tauri::command]
pub async fn polish_report(
    state: tauri::State<'_, ConfigDb>,
    content: String,
) -> Result<String, String> {
    let configs = crate::config::get_configs(state)?;

    if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        let mut prompt = String::new();
        prompt.push_str("请对以下日报进行润色和优化：\n\n");
        prompt.push_str(&content);
        prompt.push('\n');

        if let Some(tpl) = configs.ai_template.as_deref() {
            prompt.push_str("\n请按照以下要求优化：\n");
            prompt.push_str(tpl.trim());
        } else {
            prompt.push_str("\n要求：\n");
            prompt.push_str("- 保留原有的所有工作内容，包括用户补充的非开发任务\n");
            prompt.push_str("- 用第一人称描述\n");
            prompt.push_str("- 语言简洁专业\n");
            prompt.push_str("- 保持 Markdown 格式\n");
            prompt.push_str("- 只输出润色后的日报正文，不需要额外说明\n");
        }

        let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
        client.generate(&prompt)
    } else {
        Ok(content)
    }
}
