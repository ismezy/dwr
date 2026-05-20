use std::collections::HashMap;
use std::sync::OnceLock;

fn messages() -> &'static HashMap<&'static str, HashMap<&'static str, &'static str>> {
    static MESSAGES: OnceLock<HashMap<&str, HashMap<&str, &str>>> = OnceLock::new();
    MESSAGES.get_or_init(|| {
        let mut all = HashMap::new();

        let mut zh = HashMap::new();
        zh.insert("daily_report_title", "日报");
        zh.insert("weekly_report_title", "周报");
        zh.insert("no_commits_today", "今日无提交记录。");
        zh.insert("commits_label", "提交记录");
        zh.insert("changed_files", "变更文件");
        zh.insert("no_daily_reports_week", "本周暂无日报内容。");
        zh.insert("summary_daily_title", "工作汇总日报");
        zh.insert("summary_weekly_title", "工作汇总周报");
        zh.insert("no_commits_all_projects", "今日所有项目均无提交记录。");
        zh.insert("no_daily_reports_all_projects", "本周所有项目均无日报内容。");
        zh.insert("ai_daily_intro", "请根据以下 Git 提交记录生成一份工作日报。");
        zh.insert("ai_daily_project", "项目");
        zh.insert("ai_daily_date", "日期");
        zh.insert("ai_daily_commits", "提交记录");
        zh.insert("ai_recent_reports_note", "本周已生成的日报（供参考，避免重复描述同一任务）：");
        zh.insert("ai_requirements", "要求");
        zh.insert("ai_first_person", "- 用第一人称描述今天的工作内容");
        zh.insert("ai_categorize", "- 按工作内容分类汇总");
        zh.insert("ai_concise", "- 语言简洁专业");
        zh.insert("ai_markdown_only", "- 输出 Markdown 格式，只输出日报正文，不需要标题以外的额外说明");
        zh.insert("ai_avoid_duplicate_note", "注意：请对比本周已生成的日报内容，如果今天的工作任务在之前的日报中已经描述过（例如昨天写了\"新增完成A功能\"，今天只是继续完善或修复bug），请避免重复描述同一任务，只说明今天的增量进展。");
        zh.insert("ai_weekly_intro", "请根据以下本周各日的日报内容，汇总生成一份工作周报。");
        zh.insert("ai_weekly_project", "项目");
        zh.insert("ai_weekly_period", "周期");
        zh.insert("ai_weekly_daily_content", "日报内容");
        zh.insert("ai_recent_weekly_note", "前3周的周报（供参考，避免与之前的工作内容重复描述）：");
        zh.insert("ai_weekly_first_person", "- 用第一人称描述本周的工作内容");
        zh.insert("ai_weekly_summarize", "- 基于已有的日报内容，进行概括和汇总，不要遗漏重要工作");
        zh.insert("ai_weekly_categorize", "- 按项目或工作内容分类汇总");
        zh.insert("ai_weekly_avoid_duplicate", "注意：请对比前3周的周报内容，如果本周的工作任务在之前几周的周报中已经描述过，请避免重复描述同一任务，只说明本周的增量进展或新进展。");
        zh.insert("ai_polish_intro", "请对以下日报进行润色和优化：");
        zh.insert("ai_polish_optimize", "请按照以下要求优化：");
        zh.insert("ai_polish_requirements", "要求");
        zh.insert("ai_polish_retain", "- 保留原有的所有工作内容，包括用户补充的非开发任务");
        zh.insert("ai_polish_first_person", "- 用第一人称描述");
        zh.insert("ai_polish_concise", "- 语言简洁专业");
        zh.insert("ai_polish_markdown", "- 保持 Markdown 格式");
        zh.insert("ai_polish_output_only", "- 只输出润色后的日报正文，不需要额外说明");
        zh.insert("ai_summary_daily_intro", "请根据以下各项目的 Git 提交记录生成一份工作汇总日报。");
        zh.insert("ai_summary_date", "日期");
        zh.insert("ai_summary_no_commits", "今日无提交记录。");
        zh.insert("ai_summary_recent_note", "本周已生成的汇总日报（供参考，避免重复描述同一任务）：");
        zh.insert("ai_summary_requirements", "要求");
        zh.insert("ai_summary_first_person", "- 用第一人称描述今天的工作内容");
        zh.insert("ai_summary_categorize", "- 按项目或工作内容分类汇总");
        zh.insert("ai_summary_concise", "- 语言简洁专业");
        zh.insert("ai_summary_markdown_only", "- 输出 Markdown 格式，只输出日报正文，不需要标题以外的额外说明");
        zh.insert("ai_summary_avoid_duplicate", "注意：请对比本周已生成的汇总日报内容，如果今天的工作任务在之前的日报中已经描述过，请避免重复描述同一任务，只说明今天的增量进展。");
        zh.insert("ai_summary_empty_note", "注意：今天所有项目均无提交记录，请生成一份说明今日无工作记录的简短日报。");
        zh.insert("ai_summary_weekly_intro", "请根据以下各项目本周的日报内容，汇总生成一份工作周报。");
        zh.insert("ai_summary_weekly_period", "周期");
        zh.insert("ai_summary_no_daily_reports", "本周暂无日报内容。");
        zh.insert("ai_summary_recent_weekly_note", "前3周的汇总周报（供参考，避免与之前的工作内容重复描述）：");
        zh.insert("ai_summary_weekly_first_person", "- 用第一人称描述本周的工作内容");
        zh.insert("ai_summary_weekly_summarize", "- 基于各项目已有的日报内容进行概括和汇总，不要遗漏重要工作");
        zh.insert("ai_summary_weekly_categorize", "- 按项目或工作内容分类汇总");
        zh.insert("ai_summary_weekly_avoid_duplicate", "注意：请对比前3周的汇总周报内容，如果本周的工作任务在之前几周的周报中已经描述过，请避免重复描述同一任务，只说明本周的增量进展或新进展。");
        zh.insert("ai_summary_weekly_empty_note", "注意：本周所有项目均无日报内容，请生成一份说明本周无工作记录的简短周报。");
        zh.insert("ai_language_hint", "请用中文回答。");
        zh.insert("weekly_report_suffix", "周报");
        zh.insert("summary_weekly_suffix", "汇总周报");
        all.insert("zh", zh);

        let mut en = HashMap::new();
        en.insert("daily_report_title", "Daily Report");
        en.insert("weekly_report_title", "Weekly Report");
        en.insert("no_commits_today", "No commits today.");
        en.insert("commits_label", "Commits");
        en.insert("changed_files", "Changed files");
        en.insert("no_daily_reports_week", "No daily reports available for this week.");
        en.insert("summary_daily_title", "Daily Work Summary");
        en.insert("summary_weekly_title", "Weekly Work Summary");
        en.insert("no_commits_all_projects", "No commits for any project today.");
        en.insert("no_daily_reports_all_projects", "No daily reports available for any project this week.");
        en.insert("ai_daily_intro", "Please generate a work daily report based on the following Git commit logs.");
        en.insert("ai_daily_project", "Project");
        en.insert("ai_daily_date", "Date");
        en.insert("ai_daily_commits", "Commits");
        en.insert("ai_recent_reports_note", "Daily reports already generated this week (for reference, to avoid repeating the same tasks):");
        en.insert("ai_requirements", "Requirements");
        en.insert("ai_first_person", "- Describe today's work in first person");
        en.insert("ai_categorize", "- Summarize by work category");
        en.insert("ai_concise", "- Use concise and professional language");
        en.insert("ai_markdown_only", "- Output in Markdown format, only the report body, no extra explanation");
        en.insert("ai_avoid_duplicate_note", "Note: Please compare with the daily reports already generated this week. If today's tasks have been described in previous reports (e.g., yesterday's report mentioned \"completed feature A\", and today is just continuing or fixing bugs), avoid repeating the same task. Only describe today's incremental progress.");
        en.insert("ai_weekly_intro", "Please generate a work weekly report based on the following daily reports from this week.");
        en.insert("ai_weekly_project", "Project");
        en.insert("ai_weekly_period", "Period");
        en.insert("ai_weekly_daily_content", "Daily reports");
        en.insert("ai_recent_weekly_note", "Weekly reports from the past 3 weeks (for reference, to avoid repeating previous work):");
        en.insert("ai_weekly_first_person", "- Describe this week's work in first person");
        en.insert("ai_weekly_summarize", "- Summarize and generalize based on existing daily reports, do not miss important work");
        en.insert("ai_weekly_categorize", "- Summarize by project or work category");
        en.insert("ai_weekly_avoid_duplicate", "Note: Please compare with the weekly reports from the past 3 weeks. If this week's tasks have been described in previous reports, avoid repeating the same tasks. Only describe this week's incremental progress or new developments.");
        en.insert("ai_polish_intro", "Please polish and optimize the following report:");
        en.insert("ai_polish_optimize", "Please optimize according to the following requirements:");
        en.insert("ai_polish_requirements", "Requirements");
        en.insert("ai_polish_retain", "- Retain all original work content, including non-development tasks added by the user");
        en.insert("ai_polish_first_person", "- Use first person");
        en.insert("ai_polish_concise", "- Use concise and professional language");
        en.insert("ai_polish_markdown", "- Keep Markdown format");
        en.insert("ai_polish_output_only", "- Only output the polished report body, no extra explanation");
        en.insert("ai_summary_daily_intro", "Please generate a work daily summary report based on the following Git commit logs from all projects.");
        en.insert("ai_summary_date", "Date");
        en.insert("ai_summary_no_commits", "No commits today.");
        en.insert("ai_summary_recent_note", "Daily summary reports already generated this week (for reference, to avoid repeating the same tasks):");
        en.insert("ai_summary_requirements", "Requirements");
        en.insert("ai_summary_first_person", "- Describe today's work in first person");
        en.insert("ai_summary_categorize", "- Summarize by project or work category");
        en.insert("ai_summary_concise", "- Use concise and professional language");
        en.insert("ai_summary_markdown_only", "- Output in Markdown format, only the report body, no extra explanation");
        en.insert("ai_summary_avoid_duplicate", "Note: Please compare with the daily summary reports already generated this week. If today's tasks have been described in previous reports, avoid repeating the same tasks. Only describe today's incremental progress.");
        en.insert("ai_summary_empty_note", "Note: There are no commits for any project today. Please generate a brief report stating that there is no work record for today.");
        en.insert("ai_summary_weekly_intro", "Please generate a work weekly summary report based on the following daily reports from all projects this week.");
        en.insert("ai_summary_weekly_period", "Period");
        en.insert("ai_summary_no_daily_reports", "No daily reports available for this week.");
        en.insert("ai_summary_recent_weekly_note", "Weekly summary reports from the past 3 weeks (for reference, to avoid repeating previous work):");
        en.insert("ai_summary_weekly_first_person", "- Describe this week's work in first person");
        en.insert("ai_summary_weekly_summarize", "- Summarize and generalize based on existing daily reports from all projects, do not miss important work");
        en.insert("ai_summary_weekly_categorize", "- Summarize by project or work category");
        en.insert("ai_summary_weekly_avoid_duplicate", "Note: Please compare with the weekly summary reports from the past 3 weeks. If this week's tasks have been described in previous reports, avoid repeating the same tasks. Only describe this week's incremental progress or new developments.");
        en.insert("ai_summary_weekly_empty_note", "Note: There are no daily reports available for any project this week. Please generate a brief report stating that there is no work record for this week.");
        en.insert("ai_language_hint", "Please respond in English.");
        en.insert("weekly_report_suffix", "Weekly Report");
        en.insert("summary_weekly_suffix", "Weekly Summary");
        all.insert("en", en);

        all
    })
}

pub fn t<'a>(locale: &'a str, key: &'a str) -> &'a str {
    messages()
        .get(locale)
        .and_then(|m| m.get(key).copied())
        .or_else(|| messages().get("zh").and_then(|m| m.get(key).copied()))
        .unwrap_or(key)
}
