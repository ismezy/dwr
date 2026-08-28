use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_user_name: Option<String>,
    #[serde(default = "default_project_type")]
    pub project_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// 代码目录限定的分支，多个分支用逗号分隔；为空表示当前分支
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

fn default_project_type() -> String {
    "code".to_string()
}

pub fn normalize_project_type(project_type: Option<&str>) -> String {
    match project_type {
        Some("docs") => "docs".to_string(),
        _ => "code".to_string(),
    }
}

pub struct DbConnection {
    conn: Mutex<Connection>,
}

impl DbConnection {
    fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }
}

fn db_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create app data dir: {}", e))?;
    Ok(dir.join("projects.db"))
}

/// 数据结构版本：v1 = 平铺项目列表；v2 = 项目 -> 目录 两级树型结构。
/// 每次结构性数据变更递增版本号，并在 migrate_data 中追加对应分支。
const DATA_VERSION: i64 = 2;

pub fn init_db(app_handle: &AppHandle) -> Result<DbConnection, String> {
    let path = db_path(app_handle)?;
    let mut conn = Connection::open(&path).map_err(|e| format!("failed to open db: {}", e))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            code TEXT,
            path TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("failed to create table: {}", e))?;
    // 列迁移（幂等，兼容旧数据库）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN git_user_name TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN project_type TEXT NOT NULL DEFAULT 'code'",
        [],
    );
    let _ = conn.execute("ALTER TABLE projects ADD COLUMN parent_id TEXT", []);
    let _ = conn.execute("ALTER TABLE projects ADD COLUMN branch TEXT", []);
    // 按数据版本做结构性数据迁移
    migrate_data(&mut conn)?;
    Ok(DbConnection::new(conn))
}

/// 根据当前数据版本（PRAGMA user_version）逐级迁移到最新结构。
fn migrate_data(conn: &mut Connection) -> Result<(), String> {
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| format!("failed to read data version: {}", e))?;
    if version >= DATA_VERSION {
        return Ok(());
    }
    if version < 2 {
        // v2：将旧的平铺项目包装为 项目 -> 目录 两级结构
        let tx = conn
            .transaction()
            .map_err(|e| format!("failed to begin migration transaction: {}", e))?;
        migrate_flat_projects(&tx)?;
        tx.pragma_update(None, "user_version", 2)
            .map_err(|e| format!("failed to update data version: {}", e))?;
        tx.commit()
            .map_err(|e| format!("failed to commit migration: {}", e))?;
    }
    Ok(())
}

/// 把旧的平铺项目（有路径、无父级）包装到自动创建的同名项目节点下。
fn migrate_flat_projects(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id, name, code FROM projects WHERE path != '' AND parent_id IS NULL")
        .map_err(|e| format!("failed to prepare migration statement: {}", e))?;
    let legacy = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(|e| format!("failed to query legacy projects: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("failed to collect legacy projects: {}", e))?;
    for (dir_id, name, code) in legacy {
        let group_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO projects (id, name, code, path, project_type, parent_id) VALUES (?1, ?2, ?3, '', 'code', NULL)",
            params![&group_id, &name, &code],
        )
        .map_err(|e| format!("failed to insert migrated project: {}", e))?;
        conn.execute(
            "UPDATE projects SET parent_id = ?1 WHERE id = ?2",
            params![&group_id, &dir_id],
        )
        .map_err(|e| format!("failed to update migrated project: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn create_project(
    state: tauri::State<'_, DbConnection>,
    name: String,
    code: Option<String>,
    path: String,
    git_user_name: Option<String>,
    project_type: Option<String>,
    parent_id: Option<String>,
    branch: Option<String>,
) -> Result<Project, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let project_type = normalize_project_type(project_type.as_deref());
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO projects (id, name, code, path, git_user_name, project_type, parent_id, branch) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![&id, &name, &code, &path, &git_user_name, &project_type, &parent_id, &branch],
    )
    .map_err(|e| format!("failed to insert project: {}", e))?;
    Ok(Project {
        id,
        name,
        code,
        path,
        git_user_name,
        project_type,
        parent_id,
        branch,
    })
}

#[tauri::command]
pub fn get_projects(state: tauri::State<'_, DbConnection>) -> Result<Vec<Project>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, code, path, git_user_name, project_type, parent_id, branch FROM projects ORDER BY name")
        .map_err(|e| format!("failed to prepare statement: {}", e))?;
    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                path: row.get(3)?,
                git_user_name: row.get(4)?,
                project_type: row.get(5)?,
                parent_id: row.get(6)?,
                branch: row.get(7)?,
            })
        })
        .map_err(|e| format!("failed to query projects: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("failed to collect projects: {}", e))?;
    Ok(projects)
}

#[tauri::command]
pub fn update_project(
    state: tauri::State<'_, DbConnection>,
    id: String,
    name: String,
    code: Option<String>,
    path: String,
    git_user_name: Option<String>,
    project_type: Option<String>,
    parent_id: Option<String>,
    branch: Option<String>,
) -> Result<Project, String> {
    let project_type = normalize_project_type(project_type.as_deref());
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE projects SET name = ?1, code = ?2, path = ?3, git_user_name = ?4, project_type = ?5, parent_id = ?6, branch = ?7 WHERE id = ?8",
        params![&name, &code, &path, &git_user_name, &project_type, &parent_id, &branch, &id],
    )
    .map_err(|e| format!("failed to update project: {}", e))?;
    Ok(Project {
        id,
        name,
        code,
        path,
        git_user_name,
        project_type,
        parent_id,
        branch,
    })
}

#[tauri::command]
pub fn delete_project(state: tauri::State<'_, DbConnection>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let child_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM projects WHERE parent_id = ?1",
            params![&id],
            |row| row.get(0),
        )
        .map_err(|e| format!("failed to count project directories: {}", e))?;
    if child_count > 0 {
        return Err("project has directories, delete them first".to_string());
    }
    conn.execute("DELETE FROM projects WHERE id = ?1", params![&id])
        .map_err(|e| format!("failed to delete project: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一个 v1 时代的库（平铺结构），并套用 init_db 中的列迁移
    fn setup_v1_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                code TEXT,
                path TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute("ALTER TABLE projects ADD COLUMN git_user_name TEXT", [])
            .unwrap();
        conn.execute(
            "ALTER TABLE projects ADD COLUMN project_type TEXT NOT NULL DEFAULT 'code'",
            [],
        )
        .unwrap();
        conn.execute("ALTER TABLE projects ADD COLUMN parent_id TEXT", [])
            .unwrap();
        conn
    }

    #[test]
    fn migrates_flat_projects_to_tree() {
        let mut conn = setup_v1_db();
        conn.execute(
            "INSERT INTO projects (id, name, code, path) VALUES ('a', 'Alpha', 'A-1', '/repos/alpha')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO projects (id, name, path) VALUES ('b', 'Beta', '/repos/beta')",
            [],
        )
        .unwrap();

        migrate_data(&mut conn).unwrap();

        // 数据版本提升到 v2
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);

        // 原行变成目录，挂到自动创建的同名项目下
        let parent_id: String = conn
            .query_row("SELECT parent_id FROM projects WHERE id = 'a'", [], |row| {
                row.get(0)
            })
            .unwrap();
        let (name, code, path): (String, String, String) = conn
            .query_row(
                "SELECT name, code, path FROM projects WHERE id = ?1",
                params![&parent_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!((name.as_str(), code.as_str(), path.as_str()), ("Alpha", "A-1", ""));

        // 重复执行幂等：项目 2 + 目录 2，不会再次包装
        migrate_data(&mut conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn empty_db_only_bumps_version() {
        let mut conn = setup_v1_db();
        migrate_data(&mut conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
    }
}
