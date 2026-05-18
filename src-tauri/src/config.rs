use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
}

pub struct ConfigDb {
    conn: Mutex<Connection>,
}

impl ConfigDb {
    fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }
}

pub fn init_config_db(app_handle: &AppHandle) -> Result<ConfigDb, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("failed to create app data dir: {}", e))?;
    let path = dir.join("config.db");
    let conn = Connection::open(&path).map_err(|e| format!("failed to open config db: {}", e))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("failed to create config table: {}", e))?;
    Ok(ConfigDb::new(conn))
}

fn get_config_map(state: &ConfigDb) -> Result<HashMap<String, String>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM config")
        .map_err(|e| format!("failed to prepare statement: {}", e))?;
    let map = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| format!("failed to query config: {}", e))?
        .collect::<Result<HashMap<_, _>, _>>()
        .map_err(|e| format!("failed to collect config: {}", e))?;
    Ok(map)
}

fn set_config_value(state: &ConfigDb, key: &str, value: &str) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO config (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(|e| format!("failed to set config: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_configs(state: tauri::State<'_, ConfigDb>) -> Result<ConfigData, String> {
    let map = get_config_map(&state)?;
    Ok(ConfigData {
        work_dir: map.get("work_dir").cloned(),
        git_user_name: map.get("git_user_name").cloned(),
        lang: map.get("lang").cloned(),
        theme: map.get("theme").cloned(),
    })
}

#[tauri::command]
pub fn save_configs(
    state: tauri::State<'_, ConfigDb>,
    data: ConfigData,
) -> Result<(), String> {
    if let Some(ref v) = data.work_dir {
        set_config_value(&state, "work_dir", v)?;
    } else {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM config WHERE key = ?1", params!["work_dir"])
            .map_err(|e| format!("failed to delete config: {}", e))?;
    }
    if let Some(ref v) = data.git_user_name {
        set_config_value(&state, "git_user_name", v)?;
    } else {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM config WHERE key = ?1", params!["git_user_name"])
            .map_err(|e| format!("failed to delete config: {}", e))?;
    }
    if let Some(ref v) = data.lang {
        set_config_value(&state, "lang", v)?;
    } else {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM config WHERE key = ?1", params!["lang"])
            .map_err(|e| format!("failed to delete config: {}", e))?;
    }
    if let Some(ref v) = data.theme {
        set_config_value(&state, "theme", v)?;
    } else {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM config WHERE key = ?1", params!["theme"])
            .map_err(|e| format!("failed to delete config: {}", e))?;
    }
    Ok(())
}
