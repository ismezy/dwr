use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use tauri::Manager;
use crate::ai;
use crate::config::ConfigDb;
use crate::project::{DbConnection, Project};
use crate::locale;
use crate::report::{run_git_log, CommitInfo, is_valid_daily_filename, is_valid_weekly_filename, find_report_file, read_file_with_encoding, get_week_start, parse_date, format_date, next_date, prev_date, check_git_available};

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

fn build_summary_prompt(date: &str, projects_commits: &[(Project, Vec<CommitInfo>)], recent_reports: &[(String, String)], template: Option<&str>, locale: &str) -> String {
    let mut prompt = format!(
        "{}\n\n{}: {}\n\n",
        locale::t(locale, "ai_summary_daily_intro"),
        locale::t(locale, "ai_summary_date"),
        date
    );

    let mut has_commits = false;
    for (project, commits) in projects_commits {
        prompt.push_str(&format!("## {}\n", project.name));
        if commits.is_empty() {
            prompt.push_str(locale::t(locale, "ai_summary_no_commits"));
            prompt.push('\n');
        } else {
            has_commits = true;
            for (i, commit) in commits.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, commit.message));
                if !commit.files_changed.is_empty() {
                    prompt.push_str(&format!("   {}: {}\n", locale::t(locale, "changed_files"), commit.files_changed.join(", ")));
                }
            }
        }
        prompt.push('\n');
    }

    if !recent_reports.is_empty() {
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_summary_recent_note")));
        for (report_date, content) in recent_reports {
            prompt.push_str(&format!("## {}\n{}\n", report_date, content));
        }
    }

    if let Some(tpl) = template {
        prompt.push_str(tpl.trim());
    } else {
        prompt.push_str(&format!("{}:\n", locale::t(locale, "ai_summary_requirements")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_summary_first_person")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_summary_categorize")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_concise")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_markdown_only")));
    }

    if !recent_reports.is_empty() {
        prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_summary_avoid_duplicate")));
    }

    if !has_commits {
        prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_summary_empty_note")));
    }

    prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_language_hint")));

    prompt
}

fn build_weekly_summary_prompt(week_start: &str, week_end: &str, daily_reports: &[(String, String)], recent_weekly_reports: &[(String, String)], template: Option<&str>, locale: &str) -> String {
    let mut prompt = format!(
        "{}\n\n{}: {} ~ {}\n\n",
        locale::t(locale, "ai_summary_weekly_intro"),
        locale::t(locale, "ai_summary_weekly_period"),
        week_start,
        week_end
    );

    for (date, content) in daily_reports {
        prompt.push_str(&format!("## {}\n", date));
        prompt.push_str(content);
        prompt.push('\n');
    }

    if !recent_weekly_reports.is_empty() {
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_summary_recent_weekly_note")));
        for (period, content) in recent_weekly_reports {
            prompt.push_str(&format!("## {} {}\n{}\n", period, locale::t(locale, "summary_weekly_suffix"), content));
        }
    }

    if let Some(tpl) = template {
        prompt.push_str(tpl.trim());
    } else {
        prompt.push_str(&format!("{}:\n", locale::t(locale, "ai_summary_requirements")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_summary_weekly_first_person")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_summary_weekly_summarize")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_summary_weekly_categorize")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_concise")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_markdown_only")));
    }

    if !recent_weekly_reports.is_empty() {
        prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_summary_weekly_avoid_duplicate")));
    }

    if daily_reports.is_empty() {
        prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_summary_weekly_empty_note")));
    }

    prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_language_hint")));

    prompt
}

fn collect_this_week_summary_reports(
    app_handle: tauri::AppHandle,
    date: &str,
    week_start_day: i32,
    work_dir: Option<&str>,
) -> Vec<(String, String)> {
    let mut reports = Vec::new();
    let Ok(week_start) = get_week_start(date, week_start_day) else { return reports; };
    let Ok((mut y, mut m, mut d)) = parse_date(&week_start) else { return reports; };
    let Ok((end_y, end_m, end_d)) = parse_date(date) else { return reports; };

    loop {
        let date_str = format_date(y, m, d);
        if date_str != date {
            if let Ok(path) = summary_path(&date_str, work_dir, app_handle.clone()) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    reports.push((date_str, content));
                }
            }
        }
        if y == end_y && m == end_m && d == end_d {
            break;
        }
        (y, m, d) = next_date(y, m, d);
    }

    reports
}

fn collect_recent_weekly_summary_reports(
    app_handle: tauri::AppHandle,
    week_start: &str,
    work_dir: Option<&str>,
) -> Vec<(String, String)> {
    let mut reports = Vec::new();
    let Ok((mut y, mut m, mut d)) = parse_date(week_start) else { return reports; };

    for _ in 0..3 {
        for _ in 0..7 {
            (y, m, d) = prev_date(y, m, d);
        }
        let prev_week_start = format_date(y, m, d);
        let (mut ey, mut em, mut ed) = (y, m, d);
        for _ in 0..6 {
            (ey, em, ed) = next_date(ey, em, ed);
        }
        let prev_week_end = format_date(ey, em, ed);

        let dir = weekly_summary_dir(work_dir, app_handle.clone()).unwrap_or_default().join(format!("{}", y));
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("md")).unwrap_or(false) {
                    if let Ok(content) = read_file_with_encoding(&path) {
                        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        if stem.starts_with(&prev_week_start) {
                            reports.push((format!("{} ~ {}", prev_week_start, prev_week_end), content));
                        }
                    }
                }
            }
        }
    }

    reports
}

fn collect_weekly_summary_reports(
    app_handle: tauri::AppHandle,
    week_start: &str,
    week_end: &str,
    work_dir: Option<&str>,
) -> Vec<(String, String)> {
    let mut reports = Vec::new();
    let Ok((mut y, mut m, mut d)) = parse_date(week_start) else { return reports; };
    let Ok((end_y, end_m, end_d)) = parse_date(week_end) else { return reports; };

    loop {
        let date_str = format_date(y, m, d);
        if let Ok(path) = summary_path(&date_str, work_dir, app_handle.clone()) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                reports.push((date_str, content));
            }
        }
        if y == end_y && m == end_m && d == end_d {
            break;
        }
        (y, m, d) = next_date(y, m, d);
    }

    reports
}

#[tauri::command]
pub async fn generate_summary_report(
    project_state: tauri::State<'_, DbConnection>,
    config_state: tauri::State<'_, ConfigDb>,
    app_handle: tauri::AppHandle,
    date: String,
    work_dir: Option<String>,
    locale: Option<String>,
) -> Result<SummaryReport, String> {
    let locale = locale.as_deref().unwrap_or("zh");
    let configs = crate::config::get_configs(config_state)?;
    let git_path = configs.git_path.as_deref();

    // 获取所有项目目录（过滤掉树型结构中的项目节点，它们没有路径）
    let projects: Vec<Project> = crate::project::get_projects(project_state)?
        .into_iter()
        .filter(|p| !p.path.is_empty())
        .collect();

    // 仅当存在 code 类型项目时才要求 Git 可用
    if projects.iter().any(|p| p.project_type != "docs") {
        check_git_available(git_path)?;
    }

    // 收集每个项目的提交（docs 项目收集文档变更）
    let mut projects_commits: Vec<(Project, Vec<CommitInfo>)> = Vec::new();
    let mut total_commits = 0;
    for project in &projects {
        if project.project_type == "docs" {
            let changes = crate::docs::collect_doc_changes(&project.path, &project.name, &date, work_dir.as_deref());
            let commits: Vec<CommitInfo> = changes
                .iter()
                .map(|c| {
                    let label = match c.change_type.as_str() {
                        "modified" => locale::t(locale, "docs_summary_modified"),
                        "modified_no_baseline" => locale::t(locale, "docs_summary_no_baseline"),
                        _ => locale::t(locale, "docs_summary_unsupported"),
                    };
                    CommitInfo {
                        hash: String::new(),
                        message: format!("{} {}", label, c.rel_path),
                        date: date.clone(),
                        files_changed: vec![c.rel_path.clone()],
                    }
                })
                .collect();
            total_commits += commits.len();
            projects_commits.push((project.clone(), commits));
            continue;
        }
        let git_user = project.git_user_name.as_deref();
        let since = format!("{} 00:00:00", date);
        let until = format!("{} 23:59:59", date);
        let commits = run_git_log(&project.path, git_user, &since, &until, git_path).unwrap_or_default();
        total_commits += commits.len();
        projects_commits.push((project.clone(), commits));
    }

    let week_start_day = configs.week_start_day.unwrap_or(1);
    let recent_reports = collect_this_week_summary_reports(app_handle.clone(), &date, week_start_day, work_dir.as_deref());

    let content = if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        if total_commits == 0 {
            format_summary_report(&date, &projects_commits, locale)
        } else {
            let prompt = build_summary_prompt(&date, &projects_commits, &recent_reports, configs.ai_template.as_deref(), locale);
            let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
            client.generate(&prompt)?
        }
    } else {
        format_summary_report(&date, &projects_commits, locale)
    };

    let path = summary_path(&date, work_dir.as_deref(), app_handle.clone())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create summary dir: {}", e))?;
    }
    std::fs::write(&path, &content).map_err(|e| format!("failed to write summary report: {}", e))?;

    Ok(SummaryReport { date, content })
}

fn format_summary_report(_date: &str, projects_commits: &[(Project, Vec<CommitInfo>)], locale: &str) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {}", locale::t(locale, "summary_daily_title")));
    lines.push(String::new());

    let mut has_any = false;
    for (project, commits) in projects_commits {
        if !commits.is_empty() {
            has_any = true;
            lines.push(format!("## {}", project.name));
            for (i, commit) in commits.iter().enumerate() {
                lines.push(format!("{}. {}", i + 1, commit.message));
                if !commit.files_changed.is_empty() {
                    lines.push(format!("   - {}: {}", locale::t(locale, "changed_files"), commit.files_changed.join(", ")));
                }
            }
            lines.push(String::new());
        }
    }

    if !has_any {
        lines.push(locale::t(locale, "no_commits_all_projects").to_string());
    }

    lines.join("\n")
}

fn format_weekly_summary_report(week_start: &str, week_end: &str, daily_reports: &[(String, String)], locale: &str) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {} ({} ~ {})", locale::t(locale, "summary_weekly_title"), week_start, week_end));
    lines.push(String::new());

    if daily_reports.is_empty() {
        lines.push(locale::t(locale, "no_daily_reports_all_projects").to_string());
    } else {
        for (date, content) in daily_reports {
            lines.push(format!("## {}", date));
            lines.push(content.clone());
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

#[tauri::command]
pub async fn generate_weekly_summary_report(
    _project_state: tauri::State<'_, DbConnection>,
    config_state: tauri::State<'_, ConfigDb>,
    app_handle: tauri::AppHandle,
    week_start: String,
    week_end: String,
    work_dir: Option<String>,
    locale: Option<String>,
) -> Result<SummaryReport, String> {
    let locale = locale.as_deref().unwrap_or("zh");
    let daily_reports = collect_weekly_summary_reports(app_handle.clone(), &week_start, &week_end, work_dir.as_deref());

    let configs = crate::config::get_configs(config_state)?;

    let recent_weekly_reports = collect_recent_weekly_summary_reports(app_handle.clone(), &week_start, work_dir.as_deref());

    let content = if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        if daily_reports.is_empty() {
            format_weekly_summary_report(&week_start, &week_end, &daily_reports, locale)
        } else {
            let prompt = build_weekly_summary_prompt(&week_start, &week_end, &daily_reports, &recent_weekly_reports, configs.ai_template.as_deref(), locale);
            let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
            client.generate(&prompt)?
        }
    } else {
        format_weekly_summary_report(&week_start, &week_end, &daily_reports, locale)
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
        let year_name = year_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if year_name == "weekly" { continue; }
        // Scan month subdirectories: year/month/*.md
        for month_entry in std::fs::read_dir(&year_path).map_err(|e| format!("failed to read dir: {}", e))? {
            let month_entry = month_entry.map_err(|e| format!("failed to read entry: {}", e))?;
            let month_path = month_entry.path();
            if !month_path.is_dir() { continue; }
            for file_entry in std::fs::read_dir(&month_path).map_err(|e| format!("failed to read dir: {}", e))? {
                let file_entry = file_entry.map_err(|e| format!("failed to read entry: {}", e))?;
                let path = file_entry.path();
                if path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("md")).unwrap_or(false) {
                    let date = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    if is_valid_daily_filename(&date) {
                        entries.push(SummaryReportMeta {
                            date: date.clone(),
                            path: path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
        // Also scan files directly under year directory (legacy structure)
        for file_entry in std::fs::read_dir(&year_path).map_err(|e| format!("failed to read dir: {}", e))? {
            let file_entry = file_entry.map_err(|e| format!("failed to read entry: {}", e))?;
            let path = file_entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("md")).unwrap_or(false) {
                let date = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if is_valid_daily_filename(&date) {
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
    let base_dir = summary_dir(work_dir.as_deref(), app_handle)?;
    let path = find_report_file(&base_dir, &date)
        .ok_or_else(|| format!("summary report not found: {}/{}/{}.md (or legacy paths)", base_dir.display(), date.split('-').next().unwrap_or(""), date))?;
    read_file_with_encoding(&path)
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
            if path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("md")).unwrap_or(false) {
                let date = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if is_valid_weekly_filename(&date) {
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
        if path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("md")).unwrap_or(false) {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if stem.starts_with(&week_start) {
                return read_file_with_encoding(&path);
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
