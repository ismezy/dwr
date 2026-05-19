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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub week_start_day: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_template: Option<String>,
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

fn del_config_value(state: &ConfigDb, key: &str) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM config WHERE key = ?1", params![key])
        .map_err(|e| format!("failed to delete config: {}", e))?;
    Ok(())
}

fn opt_set(state: &ConfigDb, key: &str, value: &Option<String>) -> Result<(), String> {
    if let Some(ref v) = value {
        set_config_value(state, key, v)?;
    } else {
        del_config_value(state, key)?;
    }
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
        week_start_day: map.get("week_start_day").and_then(|s| s.parse().ok()),
        ai_provider: map.get("ai_provider").cloned(),
        ai_api_key: map.get("ai_api_key").cloned(),
        ai_base_url: map.get("ai_base_url").cloned(),
        ai_model: map.get("ai_model").cloned(),
        ai_template: map.get("ai_template").cloned(),
    })
}

#[tauri::command]
pub fn save_configs(
    state: tauri::State<'_, ConfigDb>,
    data: ConfigData,
) -> Result<(), String> {
    opt_set(&state, "work_dir", &data.work_dir)?;
    opt_set(&state, "git_user_name", &data.git_user_name)?;
    opt_set(&state, "lang", &data.lang)?;
    opt_set(&state, "theme", &data.theme)?;
    opt_set(&state, "week_start_day", &data.week_start_day.map(|v| v.to_string()))?;
    opt_set(&state, "ai_provider", &data.ai_provider)?;
    opt_set(&state, "ai_api_key", &data.ai_api_key)?;
    opt_set(&state, "ai_base_url", &data.ai_base_url)?;
    opt_set(&state, "ai_model", &data.ai_model)?;
    opt_set(&state, "ai_template", &data.ai_template)?;
    Ok(())
}
