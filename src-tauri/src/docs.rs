use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::report::report_dir;

/// 单文件大小上限（超过则不读取内容，仅列名）
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;
/// 提取/读取文本上限（字符数）
const MAX_TEXT_CHARS: usize = 1_000_000;
/// 新文件/无基线文件的内容摘要行数上限
const MAX_EXCERPT_LINES: usize = 100;
/// diff 摘要行数上限
const MAX_DIFF_LINES: usize = 200;

/// 直接读取的纯文本扩展名白名单
const TEXT_EXTENSIONS: &[&str] = &[
    "md", "markdown", "txt", "rst", "log", "csv", "tsv", "json", "jsonc", "yaml", "yml", "toml",
    "xml", "ini", "cfg", "conf", "properties", "html", "htm", "css", "scss", "less", "js", "jsx",
    "ts", "tsx", "mjs", "cjs", "vue", "svelte", "py", "sh", "bash", "bat", "cmd", "ps1", "java",
    "kt", "go", "rs", "c", "h", "cpp", "hpp", "cs", "sql", "php", "rb", "lua", "swift", "dart",
];

/// OOXML（zip + XML）扩展名
const OOXML_EXTENSIONS: &[&str] = &["docx", "xlsx", "pptx"];

/// 扫描时跳过的目录名（隐藏目录另行跳过）
const SKIP_DIRS: &[&str] = &["node_modules", "target", "build", "dist", "__pycache__"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocChange {
    pub rel_path: String,
    /// new / modified / modified_no_baseline / unsupported
    pub change_type: String,
    /// diff 摘要或内容摘要（unsupported 时为空）
    pub detail: String,
    /// 当前提取的纯文本，用于更新快照（不序列化）
    #[serde(skip)]
    pub new_text: Option<String>,
}

fn is_text_extension(ext: &str) -> bool {
    TEXT_EXTENSIONS.contains(&ext)
}

fn is_ooxml_extension(ext: &str) -> bool {
    OOXML_EXTENSIONS.contains(&ext)
}

/// 剥离 XML 标签取纯文本。`para_end` 为段落结束标签（如 "</w:p>"），替换为换行以保留段落结构。
fn strip_xml_tags(xml: &str, para_end: &str) -> String {
    let xml = xml.replace(para_end, "\n");
    let mut out = String::with_capacity(xml.len());
    let mut in_tag = false;
    for c in xml.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn extract_ooxml_text(path: &Path, ext: &str) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("failed to open file: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("failed to read OOXML archive: {}", e))?;

    let mut entry_names: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            entry_names.push(entry.name().to_string());
        }
    }

    let (targets, para_end): (Vec<String>, &str) = match ext {
        "docx" => (
            entry_names
                .iter()
                .filter(|n| n.as_str() == "word/document.xml")
                .cloned()
                .collect(),
            "</w:p>",
        ),
        "xlsx" => (
            entry_names
                .iter()
                .filter(|n| {
                    n.as_str() == "xl/sharedStrings.xml"
                        || (n.starts_with("xl/worksheets/") && n.ends_with(".xml"))
                })
                .cloned()
                .collect(),
            "</si>",
        ),
        "pptx" => (
            entry_names
                .iter()
                .filter(|n| n.starts_with("ppt/slides/slide") && n.ends_with(".xml"))
                .cloned()
                .collect(),
            "</a:p>",
        ),
        _ => (Vec::new(), ""),
    };

    let mut text = String::new();
    for name in targets {
        if let Ok(mut entry) = archive.by_name(&name) {
            let mut xml = String::new();
            if entry.read_to_string(&mut xml).is_ok() {
                text.push_str(&strip_xml_tags(&xml, para_end));
                text.push('\n');
            }
        }
    }
    Ok(text)
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}

fn excerpt(text: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .take(max_lines)
        .collect();
    lines.join("\n")
}

fn diff_excerpt(old: &str, new: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    let mut out = String::new();
    let mut count = 0;
    for change in diff.iter_all_changes() {
        let prefix = match change.tag() {
            ChangeTag::Insert => "+",
            ChangeTag::Delete => "-",
            ChangeTag::Equal => continue,
        };
        if count >= MAX_DIFF_LINES {
            out.push_str("...\n");
            break;
        }
        let line = change.value().trim_end_matches(['\n', '\r']);
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        out.push_str(prefix);
        out.push_str(line);
        out.push('\n');
        count += 1;
    }
    out
}

fn snapshot_dir(project_path: &str, project_name: &str, work_dir: Option<&str>) -> PathBuf {
    report_dir(project_path, project_name, work_dir).join("snapshots")
}

fn snapshot_path(base: &Path, rel_path: &str) -> PathBuf {
    base.join(format!("{}.txt", rel_path))
}

fn read_snapshot(base: &Path, rel_path: &str) -> Option<String> {
    std::fs::read_to_string(snapshot_path(base, rel_path)).ok()
}

/// 生成日报成功后调用，将本次提取的文本覆盖写入快照（每文件仅保留最新一份）。
pub fn update_snapshots(
    project_path: &str,
    project_name: &str,
    work_dir: Option<&str>,
    changes: &[DocChange],
) {
    let base = snapshot_dir(project_path, project_name, work_dir);
    for change in changes {
        if let Some(text) = &change.new_text {
            let path = snapshot_path(&base, &change.rel_path);
            if let Some(parent) = path.parent() {
                if std::fs::create_dir_all(parent).is_ok() {
                    let _ = std::fs::write(&path, text);
                }
            }
        }
    }
}

fn should_skip_dir(name: &str) -> bool {
    name.starts_with('.') || SKIP_DIRS.contains(&name)
}

fn collect_files(dir: &Path, base: &Path, target: NaiveDate, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if !should_skip_dir(&name) {
                collect_files(&path, base, target, out);
            }
            continue;
        }
        if name.starts_with('.') {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let Ok(mtime) = meta.modified() else {
            continue;
        };
        let local: DateTime<Local> = mtime.into();
        if local.date_naive() != target {
            continue;
        }
        if let Ok(rel) = path.strip_prefix(base) {
            out.push((rel.to_string_lossy().replace('\\', "/"), path));
        }
    }
}

/// 扫描 docs 型项目目录，返回目标日期内修改过的文件及其相对上份快照的变更。
pub fn collect_doc_changes(
    project_path: &str,
    project_name: &str,
    date: &str,
    work_dir: Option<&str>,
) -> Vec<DocChange> {
    let mut changes = Vec::new();
    let Ok(target) = NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
        return changes;
    };
    let root = PathBuf::from(project_path);
    let snap_base = snapshot_dir(project_path, project_name, work_dir);

    let mut files = Vec::new();
    collect_files(&root, &root, target, &mut files);
    files.sort_by(|a, b| a.0.cmp(&b.0));

    for (rel_path, path) in files {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);

        let text: Option<String> = if is_ooxml_extension(&ext) {
            if file_size > MAX_FILE_SIZE {
                None
            } else {
                extract_ooxml_text(&path, &ext).ok().map(|t| truncate_chars(&t, MAX_TEXT_CHARS))
            }
        } else if is_text_extension(&ext) {
            if file_size > MAX_FILE_SIZE {
                None
            } else {
                std::fs::read(&path)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .map(|t| truncate_chars(&t, MAX_TEXT_CHARS))
            }
        } else {
            None
        };

        let Some(text) = text else {
            changes.push(DocChange {
                rel_path,
                change_type: "unsupported".to_string(),
                detail: String::new(),
                new_text: None,
            });
            continue;
        };

        match read_snapshot(&snap_base, &rel_path) {
            None => {
                // 无基线：无法区分新旧内容，提供内容摘要
                changes.push(DocChange {
                    rel_path,
                    change_type: "modified_no_baseline".to_string(),
                    detail: excerpt(&text, MAX_EXCERPT_LINES),
                    new_text: Some(text),
                });
            }
            Some(old) if old == text => {
                // 内容未变（如 mtime 被同步工具刷新），跳过
            }
            Some(old) => {
                changes.push(DocChange {
                    rel_path,
                    change_type: "modified".to_string(),
                    detail: diff_excerpt(&old, &text),
                    new_text: Some(text),
                });
            }
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> String {
        Local::now().date_naive().format("%Y-%m-%d").to_string()
    }

    fn temp_project(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwr_docs_test_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn strip_xml_tags_extracts_paragraphs() {
        let xml = r#"<w:body><w:p><w:r><w:t>你好</w:t></w:r><w:r><w:t>世界</w:t></w:r></w:p><w:p><w:r><w:t>a &amp; b</w:t></w:r></w:p></w:body>"#;
        let text = strip_xml_tags(xml, "</w:p>");
        assert_eq!(text, "你好世界\na & b\n");
    }

    #[test]
    fn diff_excerpt_marks_added_and_removed() {
        let out = diff_excerpt("line1\nline2\n", "line1\nline3\n");
        assert!(out.contains("-line2"));
        assert!(out.contains("+line3"));
        assert!(!out.contains("line1"));
    }

    #[test]
    fn collect_changes_no_baseline_then_modified() {
        let root = temp_project("flow");
        let path = root.to_string_lossy().to_string();
        std::fs::write(root.join("note.md"), "第一行\n").unwrap();

        // 首次：无基线
        let changes = collect_doc_changes(&path, "proj", &today(), None);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, "modified_no_baseline");
        assert!(changes[0].detail.contains("第一行"));

        // 更新快照后内容未变：无变更
        update_snapshots(&path, "proj", None, &changes);
        let changes = collect_doc_changes(&path, "proj", &today(), None);
        assert!(changes.is_empty());

        // 修改文件：产生 diff
        std::fs::write(root.join("note.md"), "第一行\n第二行\n").unwrap();
        let changes = collect_doc_changes(&path, "proj", &today(), None);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change_type, "modified");
        assert!(changes[0].detail.contains("+第二行"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn collect_changes_skips_hidden_and_ignored_dirs() {
        let root = temp_project("skip");
        let path = root.to_string_lossy().to_string();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::create_dir_all(root.join("node_modules")).unwrap();
        std::fs::write(root.join(".git").join("a.md"), "x").unwrap();
        std::fs::write(root.join("node_modules").join("b.md"), "x").unwrap();
        std::fs::write(root.join("data.bin"), b"\x00\x01\x02").unwrap();

        let changes = collect_doc_changes(&path, "proj", &today(), None);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].rel_path, "data.bin");
        assert_eq!(changes[0].change_type, "unsupported");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn collect_changes_scans_nested_subdirs() {
        let root = temp_project("nested");
        let path = root.to_string_lossy().to_string();
        let nested = root.join("a").join("b").join("c");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("deep.md"), "深层内容\n").unwrap();

        let changes = collect_doc_changes(&path, "proj", &today(), None);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].rel_path, "a/b/c/deep.md");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn extract_docx_text() {
        let root = temp_project("docx");
        let docx_path = root.join("doc.docx");
        {
            let file = std::fs::File::create(&docx_path).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            writer.start_file("word/document.xml", options).unwrap();
            std::io::Write::write_all(
                &mut writer,
                "<w:body><w:p><w:r><w:t>日报内容</w:t></w:r></w:p></w:body>".as_bytes(),
            )
            .unwrap();
            writer.finish().unwrap();
        }

        let text = extract_ooxml_text(&docx_path, "docx").unwrap();
        assert!(text.contains("日报内容"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
