mod connectors;
mod core;

use std::sync::Arc;

use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

use crate::{
  connectors::telegram::runtime::TelegramRuntime,
  connectors::codex::runtime::CodexRuntime,
  core::{config_store, logbus, paths, secrets},
};

#[derive(Clone)]
struct AppState {
  config: Arc<RwLock<config_store::AppConfig>>,
  telegram: TelegramRuntime,
  codex: CodexRuntime,
  logs: logbus::LogBus,
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

  // Keep Codex runtime in sync with config changes (workspace folder influences AGENTS.md and tool context).
  state.codex.set_workspace_dir(cfg.codex.workspace_dir.clone()).await;
  state
    .codex
    .set_universal_instructions(cfg.codex.universal_instructions.clone(), cfg.codex.universal_fallback_only)
    .await;

  let path = paths::config_path(&app)?;
  config_store::save_config(&path, &cfg)
}

#[tauri::command]
async fn logs_list(state: State<'_, AppState>, limit: Option<usize>) -> Result<Vec<logbus::LogEntry>, String> {
  Ok(state.logs.list(limit.unwrap_or(200)))
}

#[tauri::command]
async fn logs_clear(state: State<'_, AppState>) -> Result<(), String> {
  state.logs.clear();
  Ok(())
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

#[tauri::command]
async fn telegram_self_test(app: AppHandle, state: State<'_, AppState>) -> Result<connectors::telegram::self_test::TelegramSelfTestResult, String> {
  let cfg = state.config.read().await.clone();
  connectors::telegram::self_test::telegram_self_test(app, cfg).await
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
      app.handle().plugin(tauri_plugin_dialog::init())?;

      let cfg = load_or_default_config(&app.handle());
      let cfg0 = cfg.clone();
      let cfg = Arc::new(RwLock::new(cfg));
      let logs = logbus::LogBus::new(1200);
      logs.push(logbus::LogLevel::Info, "app", "startup");

      let codex = CodexRuntime::new(&app.handle(), logs.clone());
      tauri::async_runtime::block_on(codex.set_workspace_dir(cfg0.codex.workspace_dir.clone()));
      tauri::async_runtime::block_on(codex.set_universal_instructions(
        cfg0.codex.universal_instructions.clone(),
        cfg0.codex.universal_fallback_only,
      ));
      let telegram = TelegramRuntime::new(cfg.clone(), codex.clone(), logs.clone());

      // Warm up Codex on startup so the UI doesn't look "stuck" and the first Telegram message is faster.
      {
        let codex2 = codex.clone();
        let logs2 = logs.clone();
        tauri::async_runtime::spawn(async move {
          logs2.push(logbus::LogLevel::Info, "codex", "startup warmup connect");
          if let Err(e) = codex2.connect().await {
            logs2.push(logbus::LogLevel::Error, "codex", format!("startup warmup failed: {e}"));
          }
        });
      }

      app.manage(AppState { config: cfg, telegram, codex, logs });
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      ping,
      get_config,
      save_config,
      logs_list,
      logs_clear,
      codex_status,
      codex_connect,
      codex_stop,
      codex_login_chatgpt,
      codex_logout,
      telegram_token_status,
      telegram_set_token,
      telegram_delete_token,
      telegram_start,
      telegram_stop,
      telegram_status,
      telegram_self_test
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
async fn codex_status(state: State<'_, AppState>) -> Result<connectors::codex::types::CodexStatus, String> {
  Ok(state.codex.status().await)
}

#[tauri::command]
async fn codex_connect(state: State<'_, AppState>) -> Result<connectors::codex::types::CodexStatus, String> {
  state.codex.connect().await?;
  Ok(state.codex.status().await)
}

#[tauri::command]
async fn codex_stop(state: State<'_, AppState>) -> Result<(), String> {
  state
    .logs
    .push(logbus::LogLevel::Warn, "codex", "codex_stop command invoked");
  state.codex.stop().await
}

#[tauri::command]
async fn codex_login_chatgpt(state: State<'_, AppState>) -> Result<connectors::codex::types::CodexStatus, String> {
  let _ = state.codex.login_chatgpt().await?;
  Ok(state.codex.status().await)
}

#[tauri::command]
async fn codex_logout(state: State<'_, AppState>) -> Result<(), String> {
  state.codex.logout().await
}
