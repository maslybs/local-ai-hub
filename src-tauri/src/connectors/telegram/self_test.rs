use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use tauri::AppHandle;

use crate::core::{config_store::AppConfig, secrets};

use super::runtime::{tg_get_me, tg_send_message};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramSelfTestResult {
  pub ok: bool,
  pub bot_username: Option<String>,
  pub sent_test_message: bool,
  pub error: Option<String>,
}

pub async fn telegram_self_test(app: AppHandle, cfg: AppConfig) -> Result<TelegramSelfTestResult, String> {
  let token = secrets::telegram_get_token(&app, cfg.telegram.token_storage)?;
  if token.trim().is_empty() {
    return Ok(TelegramSelfTestResult {
      ok: false,
      bot_username: None,
      sent_test_message: false,
      error: Some("Telegram token missing".to_string()),
    });
  }

  let client = Client::builder()
    .timeout(Duration::from_secs(20))
    .http1_only()
    .build()
    .map_err(|e| format!("reqwest client failed: {e}"))?;

  let bot_username = match tg_get_me(&client, &token).await {
    Ok(u) => u,
    Err(e) => {
      return Ok(TelegramSelfTestResult {
        ok: false,
        bot_username: None,
        sent_test_message: false,
        error: Some(e),
      });
    }
  };

  let mut sent_test_message = false;
  if let Some(&chat_id) = cfg.telegram.allowed_chat_ids.first() {
    let body = "Test: OK";
    if tg_send_message(&client, &token, chat_id, body, None).await.is_ok() {
      sent_test_message = true;
    }
  }

  Ok(TelegramSelfTestResult {
    ok: true,
    bot_username,
    sent_test_message,
    error: None,
  })
}
