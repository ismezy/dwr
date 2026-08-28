use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

use crate::ai;
use crate::config::ConfigDb;

use crate::locale;

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

pub(crate) fn report_dir(project_path: &str, project_name: &str, work_dir: Option<&str>) -> PathBuf {    if let Some(wd) = work_dir {
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

pub fn is_valid_daily_filename(stem: &str) -> bool {
    // YYYY-MM-DD
    if stem.len() != 10 { return false; }
    let bytes = stem.as_bytes();
    bytes[4] == b'-' && bytes[7] == b'-'
}

pub fn is_valid_weekly_filename(stem: &str) -> bool {
    // YYYY-MM-DD至YYYY-MM-DD
    let parts: Vec<&str> = stem.split('至').collect();
    if parts.len() != 2 { return false; }
    is_valid_daily_filename(parts[0]) && is_valid_daily_filename(parts[1])
}

pub fn read_file_with_encoding(path: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("failed to read file: {}", e))?;
    // Try UTF-8 first
    if let Ok(s) = String::from_utf8(bytes.clone()) {
        return Ok(s);
    }
    Err("file is not UTF-8 encoded. Please convert the file to UTF-8 (e.g. open in Notepad and Save As UTF-8)".to_string())
}

pub fn find_report_file(base_dir: &std::path::Path, date: &str) -> Option<std::path::PathBuf> {
    let parts: Vec<&str> = date.split('-').collect();
    let year = parts.get(0)?;
    let month = parts.get(1)?;
    // Standard path: {year}/{month}/{date}.md
    let standard = base_dir.join(year).join(month).join(format!("{}.md", date));
    if standard.exists() { return Some(standard); }
    // Fallback: {year}/{date}.md
    let fallback1 = base_dir.join(year).join(format!("{}.md", date));
    if fallback1.exists() { return Some(fallback1); }
    // Fallback: {date}.md in base_dir
    let fallback2 = base_dir.join(format!("{}.md", date));
    if fallback2.exists() { return Some(fallback2); }
    None
}

fn resolve_git_cmd(git_path: Option<&str>) -> std::process::Command {
    let mut cmd = if let Some(path) = git_path {
        if !path.is_empty() {
            Command::new(path)
        } else {
            Command::new("git")
        }
    } else {
        Command::new("git")
    };

    // Suppress the flashing console window on Windows when the GUI app spawns
    // the git child process. CREATE_NO_WINDOW = 0x08000000.
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    cmd
}

pub fn check_git_available(git_path: Option<&str>) -> Result<(), String> {
    let mut cmd = resolve_git_cmd(git_path);
    cmd.arg("--version");
    match cmd.output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => {
            if git_path.is_some() {
                Err("Git command not found at the configured path. Please check your Git path setting.".to_string())
            } else {
                Err("Git command not found. Please install Git or configure the Git path in settings.".to_string())
            }
        }
    }
}

/// 列出仓库的本地分支，供前端多选
#[tauri::command]
pub fn list_branches(
    state: tauri::State<'_, ConfigDb>,
    project_path: String,
) -> Result<Vec<String>, String> {
    let configs = crate::config::get_configs(state)?;
    let git_path = configs.git_path.as_deref();
    let mut cmd = resolve_git_cmd(git_path);
    cmd.arg("-C")
        .arg(&project_path)
        .arg("branch")
        .arg("--format=%(refname:short)");
    let output = cmd.output().map_err(|e| format!("failed to run git branch: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git branch failed: {}", stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut branches: Vec<String> = Vec::new();
    for line in stdout.lines() {
        let name = line.trim();
        if name.is_empty() || name == "HEAD" || name.ends_with("/HEAD") {
            continue;
        }
        let name = name.to_string();
        if !branches.contains(&name) {
            branches.push(name);
        }
    }
    Ok(branches)
}

/// 解析分支配置：多个分支用逗号、分号或空白分隔；为空表示不过滤（当前分支）
fn parse_branches(branch: Option<&str>) -> Vec<String> {
    branch
        .map(|s| {
            s.split([',', ';', ' ', '\t', '\n'])
                .map(|b| b.trim())
                .filter(|b| !b.is_empty())
                .map(|b| b.to_string())
                .collect()
        })
        .unwrap_or_default()
}

pub fn run_git_log(
    project_path: &str,
    git_user_name: Option<&str>,
    branch: Option<&str>,
    since: &str,
    until: &str,
    git_path: Option<&str>,
) -> Result<Vec<CommitInfo>, String> {
    let mut cmd = resolve_git_cmd(git_path);
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

    // 指定分支时按分支取提交（git 自动去重多个分支共有的提交），用 -- 避免与路径歧义
    let branches = parse_branches(branch);
    if !branches.is_empty() {
        for b in &branches {
            cmd.arg(b);
        }
        cmd.arg("--");
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
        let mut diff_cmd = resolve_git_cmd(git_path);
        diff_cmd.arg("-C")
            .arg(project_path)
            .arg("diff-tree")
            .arg("--no-commit-id")
            .arg("--name-only")
            .arg("-r")
            .arg(&commit.hash);
        let files_output = diff_cmd.output()
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

fn format_report(date: &str, project_name: &str, commits: &[CommitInfo], locale: &str) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {} {} - {}", date, locale::t(locale, "daily_report_title"), project_name));
    lines.push(String::new());

    if commits.is_empty() {
        lines.push(locale::t(locale, "no_commits_today").to_string());
    } else {
        lines.push(format!("## {} ({})", locale::t(locale, "commits_label"), commits.len()));
        lines.push(String::new());

        for (i, commit) in commits.iter().enumerate() {
            lines.push(format!("{}. {}", i + 1, commit.message));
            if !commit.files_changed.is_empty() {
                lines.push(format!("   - {}: {}", locale::t(locale, "changed_files"), commit.files_changed.join(", ")));
            }
        }
    }

    lines.join("\n")
}

fn build_docs_changes_text(changes: &[crate::docs::DocChange], locale: &str) -> String {
    if changes.is_empty() {
        return locale::t(locale, "no_doc_changes_today").to_string();
    }
    let mut text = String::new();
    for change in changes {
        match change.change_type.as_str() {
            "modified" => {
                text.push_str(&format!("### {}\n", change.rel_path));
                if !change.detail.is_empty() {
                    text.push_str(&change.detail);
                    text.push('\n');
                }
            }
            "modified_no_baseline" => {
                text.push_str(&format!(
                    "### {} {}\n",
                    change.rel_path,
                    locale::t(locale, "docs_change_no_baseline")
                ));
                if !change.detail.is_empty() {
                    text.push_str(&change.detail);
                    text.push('\n');
                }
            }
            _ => {
                text.push_str(&format!(
                    "- {} {}\n",
                    change.rel_path,
                    locale::t(locale, "docs_change_unsupported")
                ));
            }
        }
    }
    text
}

fn format_docs_report(date: &str, project_name: &str, changes: &[crate::docs::DocChange], locale: &str) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {} {} - {}", date, locale::t(locale, "daily_report_title"), project_name));
    lines.push(String::new());

    if changes.is_empty() {
        lines.push(locale::t(locale, "no_doc_changes_today").to_string());
    } else {
        lines.push(format!("## {} ({})", locale::t(locale, "docs_changes_label"), changes.len()));
        lines.push(String::new());
        for change in changes {
            lines.push(format!("- {}", change.rel_path));
        }
    }

    lines.join("\n")
}

fn build_docs_ai_prompt(date: &str, project_name: &str, changes: &[crate::docs::DocChange], recent_reports: &[(String, String)], template: Option<&str>, locale: &str) -> String {
    let changes_text = build_docs_changes_text(changes, locale);

    let mut prompt = format!(
        "{}\n\n{}: {}\n{}: {}\n\n{}:\n{}",
        locale::t(locale, "ai_docs_intro"),
        locale::t(locale, "ai_daily_project"),
        project_name,
        locale::t(locale, "ai_daily_date"),
        date,
        locale::t(locale, "ai_docs_changes"),
        changes_text
    );

    if !recent_reports.is_empty() {
        prompt.push_str(&format!("\n\n{}\n", locale::t(locale, "ai_recent_reports_note")));
        for (report_date, content) in recent_reports {
            prompt.push_str(&format!("## {}\n{}\n", report_date, content));
        }
    }

    if let Some(tpl) = template {
        prompt.push('\n');
        prompt.push_str(tpl.trim());
    } else {
        prompt.push_str(&format!("\n{}:\n", locale::t(locale, "ai_requirements")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_first_person")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_categorize")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_concise")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_markdown_only")));
    }

    if !recent_reports.is_empty() {
        prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_avoid_duplicate_note")));
    }

    prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_language_hint")));

    prompt
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

pub fn prev_date(year: i32, month: u32, day: u32) -> (i32, u32, u32) {
    if day > 1 {
        (year, month, day - 1)
    } else if month > 1 {
        let dim = days_in_month(year, month - 1);
        (year, month - 1, dim)
    } else {
        (year - 1, 12, 31)
    }
}

fn weekday(year: i32, month: u32, day: u32) -> u32 {
    // Zeller's congruence for Gregorian calendar
    // Returns 1=Monday, ..., 7=Sunday
    let mut y = year;
    let mut m = month as i32;
    if m < 3 {
        m += 12;
        y -= 1;
    }
    let q = day as i32;
    let k = y % 100;
    let j = y / 100;
    let mut h = (q + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;
    if h < 0 { h += 7; }
    // h: 0=Saturday, 1=Sunday, 2=Monday, ..., 6=Friday
    match h {
        2 => 1, // Monday
        3 => 2, // Tuesday
        4 => 3, // Wednesday
        5 => 4, // Thursday
        6 => 5, // Friday
        0 => 6, // Saturday
        1 => 7, // Sunday
        _ => 1,
    }
}

pub fn get_week_start(date: &str, week_start_day: i32) -> Result<String, String> {
    let (y, m, d) = parse_date(date)?;
    let wd = weekday(y, m, d); // 1=Monday, ..., 7=Sunday
    let target = week_start_day as u32;
    let mut diff = wd as i32 - target as i32;
    if diff < 0 { diff += 7; }
    let mut cy = y;
    let mut cm = m;
    let mut cd = d;
    for _ in 0..diff {
        (cy, cm, cd) = prev_date(cy, cm, cd);
    }
    Ok(format_date(cy, cm, cd))
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

fn collect_this_week_daily_reports(
    project_path: &str,
    project_name: &str,
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
            let path = report_path(project_path, project_name, &date_str, work_dir);
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

fn collect_recent_weekly_reports(
    project_path: &str,
    project_name: &str,
    week_start: &str,
    work_dir: Option<&str>,
) -> Vec<(String, String)> {
    let mut reports = Vec::new();
    let Ok((mut y, mut m, mut d)) = parse_date(week_start) else { return reports; };

    for _ in 0..3 {
        // Go back 7 days to get previous week start
        for _ in 0..7 {
            (y, m, d) = prev_date(y, m, d);
        }
        let prev_week_start = format_date(y, m, d);
        // Go forward 6 days to get previous week end
        let (mut ey, mut em, mut ed) = (y, m, d);
        for _ in 0..6 {
            (ey, em, ed) = next_date(ey, em, ed);
        }
        let prev_week_end = format_date(ey, em, ed);

        let dir = weekly_report_dir(project_path, project_name, work_dir).join(format!("{}", y));
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

fn format_weekly_report(week_start: &str, week_end: &str, project_name: &str, daily_reports: &[(String, String)], locale: &str) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# {} ~ {} {} - {}", week_start, week_end, locale::t(locale, "weekly_report_title"), project_name));
    lines.push(String::new());

    if daily_reports.is_empty() {
        lines.push(locale::t(locale, "no_daily_reports_week").to_string());
    } else {
        for (date, content) in daily_reports {
            lines.push(format!("## {}", date));
            lines.push(content.clone());
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

fn build_commits_text(commits: &[CommitInfo], locale: &str) -> String {
    if commits.is_empty() {
        return locale::t(locale, "no_commits_today").to_string();
    }
    let mut text = String::new();
    for (i, commit) in commits.iter().enumerate() {
        text.push_str(&format!("{}. {}\n", i + 1, commit.message));
        if !commit.files_changed.is_empty() {
            text.push_str(&format!("   {}: {}\n", locale::t(locale, "changed_files"), commit.files_changed.join(", ")));
        }
    }
    text
}

fn build_ai_prompt(date: &str, project_name: &str, commits: &[CommitInfo], recent_reports: &[(String, String)], template: Option<&str>, locale: &str) -> String {
    let commits_text = build_commits_text(commits, locale);

    let mut prompt = format!(
        "{}\n\n{}: {}\n{}: {}\n\n{}:\n{}",
        locale::t(locale, "ai_daily_intro"),
        locale::t(locale, "ai_daily_project"),
        project_name,
        locale::t(locale, "ai_daily_date"),
        date,
        locale::t(locale, "ai_daily_commits"),
        commits_text
    );

    if !recent_reports.is_empty() {
        prompt.push_str(&format!("\n\n{}\n", locale::t(locale, "ai_recent_reports_note")));
        for (report_date, content) in recent_reports {
            prompt.push_str(&format!("## {}\n{}\n", report_date, content));
        }
    }

    if let Some(tpl) = template {
        prompt.push('\n');
        prompt.push_str(tpl.trim());
    } else {
        prompt.push_str(&format!("\n{}:\n", locale::t(locale, "ai_requirements")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_first_person")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_categorize")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_concise")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_markdown_only")));
    }

    if !recent_reports.is_empty() {
        prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_avoid_duplicate_note")));
    }

    prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_language_hint")));

    prompt
}

fn build_weekly_ai_prompt(week_start: &str, week_end: &str, project_name: &str, daily_reports: &[(String, String)], recent_weekly_reports: &[(String, String)], template: Option<&str>, locale: &str) -> String {
    let mut daily_text = String::new();
    for (date, content) in daily_reports {
        daily_text.push_str(&format!("\n## {}\n{}", date, content));
    }

    let mut prompt = format!(
        "{}\n\n{}: {}\n{}: {} ~ {}\n\n{}:{}",
        locale::t(locale, "ai_weekly_intro"),
        locale::t(locale, "ai_weekly_project"),
        project_name,
        locale::t(locale, "ai_weekly_period"),
        week_start,
        week_end,
        locale::t(locale, "ai_weekly_daily_content"),
        daily_text
    );

    if !recent_weekly_reports.is_empty() {
        prompt.push_str(&format!("\n\n{}\n", locale::t(locale, "ai_recent_weekly_note")));
        for (period, content) in recent_weekly_reports {
            prompt.push_str(&format!("## {} {}\n{}\n", period, locale::t(locale, "weekly_report_suffix"), content));
        }
    }

    if let Some(tpl) = template {
        prompt.push('\n');
        prompt.push_str(tpl.trim());
    } else {
        prompt.push_str(&format!("\n{}:\n", locale::t(locale, "ai_requirements")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_weekly_first_person")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_weekly_summarize")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_categorize")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_concise")));
        prompt.push_str(&format!("{}\n", locale::t(locale, "ai_markdown_only")));
    }

    if !recent_weekly_reports.is_empty() {
        prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_weekly_avoid_duplicate")));
    }

    prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_language_hint")));

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
    locale: Option<String>,
    project_type: Option<String>,
    branch: Option<String>,
) -> Result<DailyReport, String> {
    let locale = locale.as_deref().unwrap_or("zh");
    let configs = crate::config::get_configs(state)?;

    if project_type.as_deref() == Some("docs") {
        return generate_docs_daily_report(&configs, &project_path, &project_name, &date, work_dir.as_deref(), locale);
    }

    let git_path = configs.git_path.as_deref();
    check_git_available(git_path)?;

    let since = format!("{} 00:00:00", date);
    let until = format!("{} 23:59:59", date);
    let commits = run_git_log(&project_path, git_user_name.as_deref(), branch.as_deref(), &since, &until, git_path)?;

    let week_start_day = configs.week_start_day.unwrap_or(1);
    let recent_reports = collect_this_week_daily_reports(&project_path, &project_name, &date, week_start_day, work_dir.as_deref());

    let content = if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        if commits.is_empty() {
            format_report(&date, &project_name, &commits, locale)
        } else {
            let prompt = build_ai_prompt(&date, &project_name, &commits, &recent_reports, configs.ai_template.as_deref(), locale);
            let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
            client.generate(&prompt)?
        }
    } else {
        format_report(&date, &project_name, &commits, locale)
    };

    let path = report_path(&project_path, &project_name, &date, work_dir.as_deref());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create report dir: {}", e))?;
    }
    std::fs::write(&path, &content).map_err(|e| format!("failed to write report: {}", e))?;

    Ok(DailyReport { date, content })
}

fn generate_docs_daily_report(
    configs: &crate::config::ConfigData,
    project_path: &str,
    project_name: &str,
    date: &str,
    work_dir: Option<&str>,
    locale: &str,
) -> Result<DailyReport, String> {
    let changes = crate::docs::collect_doc_changes(project_path, project_name, date, work_dir);

    let week_start_day = configs.week_start_day.unwrap_or(1);
    let recent_reports = collect_this_week_daily_reports(project_path, project_name, date, week_start_day, work_dir);

    let content = if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        if changes.is_empty() {
            format_docs_report(date, project_name, &changes, locale)
        } else {
            let prompt = build_docs_ai_prompt(date, project_name, &changes, &recent_reports, configs.ai_template.as_deref(), locale);
            let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
            client.generate(&prompt)?
        }
    } else {
        format_docs_report(date, project_name, &changes, locale)
    };

    let path = report_path(project_path, project_name, date, work_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create report dir: {}", e))?;
    }
    std::fs::write(&path, &content).map_err(|e| format!("failed to write report: {}", e))?;

    // 日报生成成功后更新快照，作为下次 diff 的基线
    crate::docs::update_snapshots(project_path, project_name, work_dir, &changes);

    Ok(DailyReport { date: date.to_string(), content })
}

#[tauri::command]
pub async fn generate_weekly_report(    state: tauri::State<'_, ConfigDb>,
    project_path: String,
    project_name: String,
    _git_user_name: Option<String>,
    week_start: String,
    week_end: String,
    work_dir: Option<String>,
    locale: Option<String>,
) -> Result<DailyReport, String> {
    let locale = locale.as_deref().unwrap_or("zh");
    let daily_reports = collect_weekly_daily_reports(&project_path, &project_name, &week_start, &week_end, work_dir.as_deref());
    let recent_weekly_reports = collect_recent_weekly_reports(&project_path, &project_name, &week_start, work_dir.as_deref());

    let configs = crate::config::get_configs(state)?;

    let content = if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        if daily_reports.is_empty() {
            format_weekly_report(&week_start, &week_end, &project_name, &daily_reports, locale)
        } else {
            let prompt = build_weekly_ai_prompt(&week_start, &week_end, &project_name, &daily_reports, &recent_weekly_reports, configs.ai_template.as_deref(), locale);
            let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
            client.generate(&prompt)?
        }
    } else {
        format_weekly_report(&week_start, &week_end, &project_name, &daily_reports, locale)
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
                        entries.push(ReportMeta {
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
    let base_dir = report_dir(&project_path, &project_name, work_dir.as_deref());
    let path = find_report_file(&base_dir, &date)
        .ok_or_else(|| format!("report not found: {}/{}/{}.md (or legacy paths)", base_dir.display(), date.split('-').next().unwrap_or(""), date))?;
    read_file_with_encoding(&path)
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
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create report dir: {}", e))?;
    }
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
            if path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("md")).unwrap_or(false) {
                let date = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if is_valid_weekly_filename(&date) {
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
    locale: Option<String>,
) -> Result<String, String> {
    let locale = locale.as_deref().unwrap_or("zh");
    let configs = crate::config::get_configs(state)?;

    if let (Some(provider), Some(api_key), Some(model)) =
        (configs.ai_provider.as_deref(), configs.ai_api_key.as_deref(), configs.ai_model.as_deref())
    {
        let mut prompt = String::new();
        prompt.push_str(&format!("{}\n\n", locale::t(locale, "ai_polish_intro")));
        prompt.push_str(&content);
        prompt.push('\n');

        if let Some(tpl) = configs.ai_template.as_deref() {
            prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_polish_optimize")));
            prompt.push_str(tpl.trim());
        } else {
            prompt.push_str(&format!("\n{}:\n", locale::t(locale, "ai_polish_requirements")));
            prompt.push_str(&format!("{}\n", locale::t(locale, "ai_polish_retain")));
            prompt.push_str(&format!("{}\n", locale::t(locale, "ai_polish_first_person")));
            prompt.push_str(&format!("{}\n", locale::t(locale, "ai_polish_concise")));
            prompt.push_str(&format!("{}\n", locale::t(locale, "ai_polish_markdown")));
            prompt.push_str(&format!("{}\n", locale::t(locale, "ai_polish_output_only")));
        }

        prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_language_hint")));

        let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
        client.generate(&prompt)
    } else {
        Ok(content)
    }
}

#[tauri::command]
pub async fn refine_report(
    state: tauri::State<'_, ConfigDb>,
    content: String,
    instruction: String,
    locale: Option<String>,
) -> Result<String, String> {
    let locale = locale.as_deref().unwrap_or("zh");
    let instruction = instruction.trim();
    if instruction.is_empty() {
        return Ok(content);
    }

    let configs = crate::config::get_configs(state)?;
    let (provider, api_key, model) = match (
        configs.ai_provider.as_deref(),
        configs.ai_api_key.as_deref(),
        configs.ai_model.as_deref(),
    ) {
        (Some(p), Some(k), Some(m)) => (p, k, m),
        _ => return Err("AI not configured".to_string()),
    };

    let mut prompt = String::new();
    prompt.push_str(&format!("{}\n\n", locale::t(locale, "ai_refine_intro")));
    prompt.push_str(&format!("## {}\n", locale::t(locale, "ai_refine_original")));
    prompt.push_str(&content);
    prompt.push_str("\n\n");
    prompt.push_str(&format!("## {}\n", locale::t(locale, "ai_refine_user_instruction")));
    prompt.push_str(instruction);
    prompt.push_str("\n\n");
    prompt.push_str(&format!("{}:\n", locale::t(locale, "ai_refine_requirements")));
    prompt.push_str(&format!("{}\n", locale::t(locale, "ai_refine_retain")));
    prompt.push_str(&format!("{}\n", locale::t(locale, "ai_refine_keep_structure")));
    prompt.push_str(&format!("{}\n", locale::t(locale, "ai_refine_markdown")));
    prompt.push_str(&format!("{}\n", locale::t(locale, "ai_refine_output_only")));
    prompt.push_str(&format!("\n{}\n", locale::t(locale, "ai_language_hint")));

    let client = ai::create_client(provider, api_key, configs.ai_base_url.as_deref(), model)?;
    client.generate(&prompt)
}
