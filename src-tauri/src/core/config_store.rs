use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenStorageMode {
  Keychain,
  File,
}

impl Default for TokenStorageMode {
  fn default() -> Self {
    Self::Keychain
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
  #[serde(default)]
  pub allowed_chat_ids: Vec<i64>,
  #[serde(default = "default_poll_timeout_sec")]
  pub poll_timeout_sec: u64,
  #[serde(default)]
  pub token_storage: TokenStorageMode,
}

fn default_poll_timeout_sec() -> u64 {
  20
}

impl Default for TelegramConfig {
  fn default() -> Self {
    Self {
      allowed_chat_ids: vec![],
      poll_timeout_sec: default_poll_timeout_sec(),
      token_storage: TokenStorageMode::default(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
  // Folder that Codex should treat as the "workspace" (used for AGENTS.md and tool context).
  // If not set, we fall back to current working directory of the app process.
  #[serde(default)]
  pub workspace_dir: Option<String>,

  // Used when the workspace folder has no AGENTS.md / AGENTS.override.md.
  // Keep it short and product-oriented; avoid listing file paths or internal details.
  #[serde(default)]
  pub universal_instructions: String,

  // If true, universal_instructions are only applied when the workspace has no AGENTS.md.
  // If false, universal_instructions always apply (as a "global" baseline).
  #[serde(default = "default_universal_fallback_only")]
  pub universal_fallback_only: bool,
}

impl Default for CodexConfig {
  fn default() -> Self {
    Self {
      workspace_dir: None,
      universal_instructions: String::new(),
      universal_fallback_only: default_universal_fallback_only(),
    }
  }
}

fn default_universal_fallback_only() -> bool {
  true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
  // User-selected language code (e.g. "en", "uk").
  // If not set, the frontend should use the system language.
  #[serde(default)]
  pub language: Option<String>,
}

impl Default for UiConfig {
  fn default() -> Self {
    Self { language: None }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
  #[serde(default)]
  pub telegram: TelegramConfig,
  #[serde(default)]
  pub codex: CodexConfig,
  #[serde(default)]
  pub ui: UiConfig,
}

pub fn load_config(path: &PathBuf) -> Result<AppConfig, String> {
  if !path.exists() {
    return Ok(AppConfig::default());
  }
  let raw = fs::read_to_string(path).map_err(|e| format!("read config failed: {e}"))?;
  serde_json::from_str(&raw).map_err(|e| format!("parse config failed: {e}"))
}

pub fn save_config(path: &PathBuf, cfg: &AppConfig) -> Result<(), String> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|e| format!("create config dir failed: {e}"))?;
  }
  let raw = serde_json::to_string_pretty(cfg).map_err(|e| format!("serialize config failed: {e}"))?;
  fs::write(path, raw).map_err(|e| format!("write config failed: {e}"))
}
