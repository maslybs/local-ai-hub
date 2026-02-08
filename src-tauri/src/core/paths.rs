use std::path::PathBuf;

use tauri::{AppHandle, Manager};

pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
  app
    .path()
    .app_data_dir()
    .map_err(|e| format!("app_data_dir error: {e}"))
}

pub fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
  Ok(app_data_dir(app)?.join("config.json"))
}

pub fn telegram_bot_state_path(app: &AppHandle) -> Result<PathBuf, String> {
  Ok(app_data_dir(app)?.join("bot-state.json"))
}

pub fn telegram_token_fallback_path(app: &AppHandle) -> Result<PathBuf, String> {
  Ok(app_data_dir(app)?.join("telegram-token.txt"))
}

pub fn codex_chat_threads_path(app: &AppHandle) -> Result<PathBuf, String> {
  Ok(app_data_dir(app)?.join("codex-chat-threads.json"))
}

pub fn codex_home_dir(app: &AppHandle) -> Result<PathBuf, String> {
  Ok(app_data_dir(app)?.join("codex-home"))
}

pub fn codex_tools_dir(app: &AppHandle) -> Result<PathBuf, String> {
  Ok(app_data_dir(app)?.join("codex-tools"))
}

pub fn codex_global_agents_override_path(app: &AppHandle) -> Result<PathBuf, String> {
  Ok(codex_home_dir(app)?.join("AGENTS.override.md"))
}
