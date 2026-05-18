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

pub fn init_db(app_handle: &AppHandle) -> Result<DbConnection, String> {
    let path = db_path(app_handle)?;
    let conn = Connection::open(&path).map_err(|e| format!("failed to open db: {}", e))?;
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
    // 迁移：添加 git_user_name 列（兼容旧数据库）
    let _ = conn.execute(
        "ALTER TABLE projects ADD COLUMN git_user_name TEXT",
        [],
    );
    Ok(DbConnection::new(conn))
}

#[tauri::command]
pub fn create_project(
    state: tauri::State<'_, DbConnection>,
    name: String,
    code: Option<String>,
    path: String,
    git_user_name: Option<String>,
) -> Result<Project, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO projects (id, name, code, path, git_user_name) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&id, &name, &code, &path, &git_user_name],
    )
    .map_err(|e| format!("failed to insert project: {}", e))?;
    Ok(Project {
        id,
        name,
        code,
        path,
        git_user_name,
    })
}

#[tauri::command]
pub fn get_projects(state: tauri::State<'_, DbConnection>) -> Result<Vec<Project>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, code, path, git_user_name FROM projects ORDER BY name")
        .map_err(|e| format!("failed to prepare statement: {}", e))?;
    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                path: row.get(3)?,
                git_user_name: row.get(4)?,
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
) -> Result<Project, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE projects SET name = ?1, code = ?2, path = ?3, git_user_name = ?4 WHERE id = ?5",
        params![&name, &code, &path, &git_user_name, &id],
    )
    .map_err(|e| format!("failed to update project: {}", e))?;
    Ok(Project {
        id,
        name,
        code,
        path,
        git_user_name,
    })
}

#[tauri::command]
pub fn delete_project(state: tauri::State<'_, DbConnection>, id: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM projects WHERE id = ?1", params![&id])
        .map_err(|e| format!("failed to delete project: {}", e))?;
    Ok(())
}
