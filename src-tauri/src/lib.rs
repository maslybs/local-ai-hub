mod connectors;
mod core;

use std::sync::Arc;

use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use crate::{
  connectors::telegram::runtime::TelegramRuntime,
  connectors::codex::runtime::CodexRuntime,
  core::{config_store, paths, secrets},
};

#[derive(Clone)]
struct AppState {
  config: Arc<RwLock<config_store::AppConfig>>,
  telegram: TelegramRuntime,
  codex: CodexRuntime,
}

#[tauri::command]
async fn ping() -> String {
  "pong".to_string()
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<config_store::AppConfig, String> {
  Ok(state.config.read().await.clone())
}

#[tauri::command]
async fn save_config(app: AppHandle, state: State<'_, AppState>, cfg: config_store::AppConfig) -> Result<(), String> {
  {
    let mut guard = state.config.write().await;
    *guard = cfg.clone();
  }
  let path = paths::config_path(&app)?;
  config_store::save_config(&path, &cfg)
}

#[tauri::command]
async fn telegram_token_status(app: AppHandle, state: State<'_, AppState>) -> Result<secrets::SecretStatus, String> {
  let mode = state.config.read().await.telegram.token_storage;
  secrets::telegram_token_status(&app, mode)
}

#[tauri::command]
async fn telegram_set_token(app: AppHandle, state: State<'_, AppState>, token: String) -> Result<secrets::SecretStatus, String> {
  let mode = state.config.read().await.telegram.token_storage;
  secrets::telegram_set_token_for_mode(&app, mode, &token)?;
  let mut st = secrets::telegram_token_status(&app, mode)?;
  if !st.stored && st.error.is_none() {
    st.error = Some("Token was not found after saving. Try enabling File (fallback).".to_string());
  }
  Ok(st)
}

#[tauri::command]
async fn telegram_delete_token(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
  let mode = state.config.read().await.telegram.token_storage;
  secrets::telegram_delete_token(&app, mode)
}

#[tauri::command]
async fn telegram_start(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
  state.telegram.start(app).await
}

#[tauri::command]
async fn telegram_stop(state: State<'_, AppState>) -> Result<(), String> {
  state.telegram.stop().await
}

#[tauri::command]
async fn telegram_status(state: State<'_, AppState>) -> Result<connectors::telegram::types::TelegramStatus, String> {
  Ok(state.telegram.status().await)
}

fn load_or_default_config(app: &AppHandle) -> config_store::AppConfig {
  let path = paths::config_path(app).ok();
  if let Some(path) = path {
    config_store::load_config(&path).unwrap_or_default()
  } else {
    config_store::AppConfig::default()
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // Always enable logging; for release it will go to file.
      app.handle().plugin(
        tauri_plugin_log::Builder::default()
          .level(log::LevelFilter::Info)
          .build(),
      )?;

      let cfg = load_or_default_config(&app.handle());
      let cfg = Arc::new(RwLock::new(cfg));
      let codex = CodexRuntime::new();
      let telegram = TelegramRuntime::new(cfg.clone(), codex.clone());

      app.manage(AppState { config: cfg, telegram, codex });
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      ping,
      get_config,
      save_config,
      codex_status,
      telegram_token_status,
      telegram_set_token,
      telegram_delete_token,
      telegram_start,
      telegram_stop,
      telegram_status
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
async fn codex_status(state: State<'_, AppState>) -> Result<connectors::codex::types::CodexStatus, String> {
  Ok(state.codex.status().await)
}
