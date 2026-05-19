use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use tauri::Manager;
use crate::ai;
use crate::config::ConfigDb;
use crate::project::{DbConnection, Project};
use crate::report::{run_git_log, CommitInfo, collect_weekly_daily_reports};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryReport {
    pub date: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryReportMeta {
    pub date: String,
    pub path: String,
}

fn summary_dir(work_dir: Option<&str>, app_handle: tauri::AppHandle) -> Result<PathBuf, String> {
    if let Some(wd) = work_dir {
        Ok(PathBuf::from(wd).join("_summary"))
    } else {
        let dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("failed to get app data dir: {}", e))?;
        Ok(dir.join("summary"))
    }
}

fn summary_path(date: &str, work_dir: Option<&str>, app_handle: tauri::AppHandle) -> Result<PathBuf, String> {
    let parts: Vec<&str> = date.split('-').collect();
    let year = parts.get(0).unwrap_or(&"");
    let month = parts.get(1).unwrap_or(&"");
    Ok(summary_dir(work_dir, app_handle)?.join(year).join(month).join(format!("{}.md", date)))
}

fn weekly_summary_dir(work_dir: Option<&str>, app_handle: tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(summary_dir(work_dir, app_handle)?.join("weekly"))
}

fn weekly_summary_path(week_start: &str, week_end: &str, work_dir: Option<&str>, app_handle: tauri::AppHandle) -> Result<PathBuf, String> {
    let year = week_start.split('-').next().unwrap_or("");
    Ok(weekly_summary_dir(work_dir, app_handle)?.join(year).join(format!("{}至{}.md", week_start, week_end)))
}

fn build_summary_prompt(date: &str, projects_commits: &[(Project, Vec<CommitInfo>)], template: Option<&str>) -> String {
    let mut prompt = format!(
        "请根据以下各项目的 Git 提交记录生成一份工作汇总日报。\n\n日期：{}\n\n",
        date
    );

    let mut has_commits = false;
    for (project, commits) in projects_commits {
        prompt.push_str(&format!("## {}\n", project.name));
        if commits.is_empty() {
            prompt.push_str("今日无提交记录。\n");
        } else {
            has_commits = true;
            for (i, commit) in commits.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, commit.message));
                if !commit.files_changed.is_empty() {
                    prompt.push_str(&format!("   变更文件: {}\n", commit.files_changed.join(", ")));
                }
            }
        }
        prompt.push('\n');
    }

    if let Some(tpl) = template {
        prompt.push_str(tpl.trim());
    } else {
        prompt.push_str("要求：\n");
        prompt.push_str("- 用第一人称描述今天的工作内容\n");
        prompt.push_str("- 按项目或工作内容分类汇总\n");
        prompt.push_str("- 语言简洁专业\n");
        prompt.push_str("- 输出 Markdown 格式，只输出日报正文，不需要标题以外的额外说明\n");
    }

    // 如果所有项目都没有提交，在 prompt 里标注一下，但 AI 仍然会被调用（如果配置了）
    // 不过我们在调用前已经检查了 has_commits
    if !has_commits {
        prompt.push_str("\n注意：今天所有项目均无提交记录，请生成一份说明今日无工作记录的简短日报。\n");
    }

    prompt
}

fn build_weekly_summary_prompt(week_start: &str, week_end: &str, projects_reports: &[(Project, Vec<(String, String)>)], template: Option<&str>) -> String {
    let mut prompt = format!(
        "请根据以下各项目本周的日报内容，汇总生成一份工作周报。\n\n周期：{} ~ {}\n\n",
        week_start, week_end
    );

    let mut has_any = false;
    for (project, daily_reports) in projects_reports {
        prompt.push_str(&format!("## {}\n", project.name));
        if daily_reports.is_empty() {
            prompt.push_str("本周暂无日报内容。\n");
        } else {
            has_any = true;
            for (date, content) in daily_reports {
                prompt.push_str(&format!("### {}\n{}", date, content));
                prompt.push('\n');
            }
        }
        prompt.push('\n');
    }

    if let Some(tpl) = template {
        prompt.push_str(tpl.trim());
    } else {
        prompt.push_str("要求：\n");
        prompt.push_str("- 用第一人称描述本周的工作内容\n");
        prompt.push_str("- 基于各项目已有的日报内容进行概括和汇总，不要遗漏重要工作\n");
        prompt.push_str("- 按项目或工作内容分类汇总\n");
        prompt.push_str("- 语言简洁专业\n");
        prompt.push_str("- 输出 Markdown 格式，只输出周报正文，不需要标题以外的额外说明\n");
    }

    if !has_any {
        prompt.push_str("\n注意：本周所有项目均无日报内容，请生成一份说明本周无工作记录的简短周报。\n");
    }

    prompt
}

#[tauri::command]
pub async fn generate_summary_report(
    project_state: tauri::State<'_, DbConnection>,
    config_state: tauri::State<'_, ConfigDb>,
    app_handle: tauri::AppHandle,
    date: String,
    work_dir: Option<String>,
) -> Result<SummaryReport, String> {
    // 获取所有项目
    let projects = crate::project::get_projects(project_state)?;

    // 收集每个项目的提交
    let mut projects_commits: Vec<(Project, Vec<CommitInfo>)> = Vec::new();
    let mut total_commits = 0;
    for project in &projects {
        let git_user = project.git_user_name.as_deref();
        let since = format!("{} 00:00:00", date);
        let until = format!("{} 23:59:59", date);
        let commits = run_git_log(&project.path, git_user, &since, &until).unwrap_or_default();
        total_commits += commits.len();
        projects_commits.push((project.clone(), commits));
    }

    let configs = crate::config::get_configs(config_state)?;

    let content = if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        if total_commits == 0 {
            format_summary_report(&date, &projects_commits)
        } else {
            let prompt = build_summary_prompt(&date, &projects_commits, configs.ai_template.as_deref());
            let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
            client.generate(&prompt)?
        }
    } else {
        format_summary_report(&date, &projects_commits)
    };

    let path = summary_path(&date, work_dir.as_deref(), app_handle.clone())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create summary dir: {}", e))?;
    }
    std::fs::write(&path, &content).map_err(|e| format!("failed to write summary report: {}", e))?;

    Ok(SummaryReport { date, content })
}

fn format_summary_report(_date: &str, projects_commits: &[(Project, Vec<CommitInfo>)]) -> String {
    let mut lines = Vec::new();
    lines.push("# 工作汇总日报".to_string());
    lines.push(String::new());

    let mut has_any = false;
    for (project, commits) in projects_commits {
        if !commits.is_empty() {
            has_any = true;
            lines.push(format!("## {}", project.name));
            for (i, commit) in commits.iter().enumerate() {
                lines.push(format!("{}. {}", i + 1, commit.message));
                if !commit.files_changed.is_empty() {
                    lines.push(format!("   - 变更文件: {}", commit.files_changed.join(", ")));
                }
            }
            lines.push(String::new());
        }
    }

    if !has_any {
        lines.push("今日所有项目均无提交记录。".to_string());
    }

    lines.join("\n")
}

fn format_weekly_summary_report(week_start: &str, week_end: &str, projects_reports: &[(Project, Vec<(String, String)>)]) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# 工作汇总周报 ({} ~ {})", week_start, week_end));
    lines.push(String::new());

    let mut has_any = false;
    for (project, daily_reports) in projects_reports {
        if !daily_reports.is_empty() {
            has_any = true;
            lines.push(format!("## {}", project.name));
            for (date, content) in daily_reports {
                lines.push(format!("### {}", date));
                lines.push(content.clone());
            }
            lines.push(String::new());
        }
    }

    if !has_any {
        lines.push("本周所有项目均无日报内容。".to_string());
    }

    lines.join("\n")
}

#[tauri::command]
pub async fn generate_weekly_summary_report(
    project_state: tauri::State<'_, DbConnection>,
    config_state: tauri::State<'_, ConfigDb>,
    app_handle: tauri::AppHandle,
    week_start: String,
    week_end: String,
    work_dir: Option<String>,
) -> Result<SummaryReport, String> {
    let projects = crate::project::get_projects(project_state)?;

    let mut projects_reports: Vec<(Project, Vec<(String, String)>)> = Vec::new();
    let mut total_reports = 0;
    for project in &projects {
        let reports = collect_weekly_daily_reports(&project.path, &project.name, &week_start, &week_end, work_dir.as_deref());
        total_reports += reports.len();
        projects_reports.push((project.clone(), reports));
    }

    let configs = crate::config::get_configs(config_state)?;

    let content = if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        if total_reports == 0 {
            format_weekly_summary_report(&week_start, &week_end, &projects_reports)
        } else {
            let prompt = build_weekly_summary_prompt(&week_start, &week_end, &projects_reports, configs.ai_template.as_deref());
            let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
            client.generate(&prompt)?
        }
    } else {
        format_weekly_summary_report(&week_start, &week_end, &projects_reports)
    };

    let dir = weekly_summary_dir(work_dir.as_deref(), app_handle.clone())?.join(week_start.split('-').next().unwrap_or(""));
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create weekly summary dir: {}", e))?;

    let path = weekly_summary_path(&week_start, &week_end, work_dir.as_deref(), app_handle.clone())?;
    std::fs::write(&path, &content).map_err(|e| format!("failed to write weekly summary report: {}", e))?;

    Ok(SummaryReport { date: week_start, content })
}

#[tauri::command]
pub fn get_summary_report_list(
    app_handle: tauri::AppHandle,
    work_dir: Option<String>,
) -> Result<Vec<SummaryReportMeta>, String> {
    let base_dir = summary_dir(work_dir.as_deref(), app_handle.clone())?;
    if !base_dir.exists() {
        std::fs::create_dir_all(&base_dir).map_err(|e| format!("failed to create summary dir: {}", e))?;
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
                    entries.push(SummaryReportMeta {
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
pub fn read_summary_report(
    app_handle: tauri::AppHandle,
    date: String,
    work_dir: Option<String>,
) -> Result<String, String> {
    let path = summary_path(&date, work_dir.as_deref(), app_handle)?;
    if !path.exists() {
        return Err("report not found".to_string());
    }
    std::fs::read_to_string(&path).map_err(|e| format!("failed to read summary report: {}", e))
}

#[tauri::command]
pub fn save_summary_report(
    app_handle: tauri::AppHandle,
    date: String,
    content: String,
    work_dir: Option<String>,
) -> Result<(), String> {
    let path = summary_path(&date, work_dir.as_deref(), app_handle)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create summary dir: {}", e))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("failed to save summary report: {}", e))
}

#[tauri::command]
pub fn get_weekly_summary_report_list(
    app_handle: tauri::AppHandle,
    work_dir: Option<String>,
) -> Result<Vec<SummaryReportMeta>, String> {
    let base_dir = weekly_summary_dir(work_dir.as_deref(), app_handle.clone())?;
    if !base_dir.exists() {
        std::fs::create_dir_all(&base_dir).map_err(|e| format!("failed to create weekly summary dir: {}", e))?;
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
                entries.push(SummaryReportMeta {
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
pub fn read_weekly_summary_report(
    app_handle: tauri::AppHandle,
    week_start: String,
    work_dir: Option<String>,
) -> Result<String, String> {
    let year = week_start.split('-').next().unwrap_or("");
    let dir = weekly_summary_dir(work_dir.as_deref(), app_handle.clone())?.join(year);
    if !dir.exists() {
        return Err("report not found".to_string());
    }
    for entry in std::fs::read_dir(&dir).map_err(|e| format!("failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if stem.starts_with(&week_start) {
                return std::fs::read_to_string(&path).map_err(|e| format!("failed to read weekly summary report: {}", e));
            }
        }
    }
    Err("report not found".to_string())
}

#[tauri::command]
pub fn save_weekly_summary_report(
    app_handle: tauri::AppHandle,
    week_start: String,
    content: String,
    work_dir: Option<String>,
) -> Result<(), String> {
    let year = week_start.split('-').next().unwrap_or("");
    let dir = weekly_summary_dir(work_dir.as_deref(), app_handle.clone())?.join(year);
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create weekly summary dir: {}", e))?;

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
        std::fs::write(&path, content).map_err(|e| format!("failed to save weekly summary report: {}", e))
    } else {
        Err("report not found".to_string())
    }
}
