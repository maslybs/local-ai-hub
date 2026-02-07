use std::{fs, sync::Arc, time::Duration};

use reqwest::Client;
use serde::Deserialize;
use tauri::AppHandle;
use tokio::sync::{watch, RwLock};

use crate::core::{config_store::AppConfig, logbus, paths, secrets};
use crate::connectors::codex::runtime::CodexRuntime;

use super::types::{BotState, TelegramStatus};

#[derive(Clone)]
pub struct TelegramRuntime {
  inner: Arc<Inner>,
}

struct Inner {
  status: RwLock<TelegramStatus>,
  stop_tx: RwLock<Option<watch::Sender<bool>>>,
  config: Arc<RwLock<AppConfig>>,
  codex: CodexRuntime,
  logs: logbus::LogBus,
}

impl TelegramRuntime {
  pub fn new(config: Arc<RwLock<AppConfig>>, codex: CodexRuntime, logs: logbus::LogBus) -> Self {
    Self {
      inner: Arc::new(Inner {
        status: RwLock::new(TelegramStatus::default()),
        stop_tx: RwLock::new(None),
        config,
        codex,
        logs,
      }),
    }
  }

  pub async fn status(&self) -> TelegramStatus {
    self.inner.status.read().await.clone()
  }

  pub async fn start(&self, app: AppHandle) -> Result<(), String> {
    // idempotent start
    if self.inner.status.read().await.running {
      return Ok(());
    }

    let cfg0 = self.inner.config.read().await.clone();
    let token = secrets::telegram_get_token(&app, cfg0.telegram.token_storage)?;
    if token.trim().is_empty() {
      return Err("Telegram token missing".to_string());
    }

    self.inner.logs.push(logbus::LogLevel::Info, "telegram", "start polling");
    let (tx, mut rx) = watch::channel(false);
    *self.inner.stop_tx.write().await = Some(tx);

    {
      let mut st = self.inner.status.write().await;
      st.running = true;
      st.last_error = None;
    }

    let runtime = self.clone();
    tauri::async_runtime::spawn(async move {
      let client = Client::builder()
        .timeout(Duration::from_secs(70))
        .build()
        .expect("reqwest client");

      // Warm up Codex in the background so the first user message feels faster.
      {
        let codex = runtime.inner.codex.clone();
        let logs = runtime.inner.logs.clone();
        tauri::async_runtime::spawn(async move {
          logs.push(logbus::LogLevel::Info, "codex", "warmup connect");
          if let Err(e) = codex.connect().await {
            logs.push(logbus::LogLevel::Error, "codex", format!("warmup failed: {e}"));
          }
        });
      }

      // get bot username
      match tg_get_me(&client, &token).await {
        Ok(username) => {
          let mut st = runtime.inner.status.write().await;
          st.bot_username = username;
        }
        Err(e) => {
          let mut st = runtime.inner.status.write().await;
          st.last_error = Some(e);
          let msg = st.last_error.clone().unwrap_or_default();
          runtime
            .inner
            .logs
            .push(logbus::LogLevel::Error, "telegram", format!("getMe failed: {msg}"));
        }
      }

      let mut state = match load_bot_state(&app) {
        Ok(s) => s,
        Err(e) => {
          let mut st = runtime.inner.status.write().await;
          st.last_error = Some(e);
          BotState::default()
        }
      };

      loop {
        if *rx.borrow() {
          break;
        }

        let cfg = runtime.inner.config.read().await.clone();
        let poll_timeout = cfg.telegram.poll_timeout_sec.max(1).min(60) as i64;

        let updates_fut = tg_get_updates(&client, &token, state.offset, poll_timeout);
        let updates = tokio::select! {
          _ = rx.changed() => break,
          res = updates_fut => res,
        };

        match updates {
          Ok((new_offset, items)) => {
            {
              let mut st = runtime.inner.status.write().await;
              st.last_poll_unix_ms = Some(now_unix_ms());
              st.last_error = None;
            }
            runtime
              .inner
              .logs
              .push(logbus::LogLevel::Info, "telegram", format!("poll ok: {} updates", items.len()));

            // process messages
            for msg in items {
              if let Some((chat_id, message_id, text)) = extract_text_message(&msg) {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                  continue;
                }

                let (cmd, rest) = parse_command(trimmed);
                runtime
                  .inner
                  .logs
                  .push(logbus::LogLevel::Info, "telegram", format!("msg chat_id={chat_id} cmd={}", cmd.clone().unwrap_or_else(|| "(text)".to_string())));
                match cmd.as_deref() {
                  Some("/start") => {
                    let body = "Бот підключено.\n\nКоманди:\n/whoami\n/ping";
                    if let Err(e) = tg_send_message(&client, &token, chat_id, body, Some(message_id)).await {
                      log::info!("telegram: send /start reply failed: {e}");
                    }
                  }
                  Some("/whoami") => {
                    let body = format!("chat_id: {chat_id}");
                    if let Err(e) = tg_send_message(&client, &token, chat_id, &body, Some(message_id)).await {
                      log::info!("telegram: send /whoami reply failed: {e}");
                    }
                  }
                  Some("/ping") => {
                    let allowed = cfg.telegram.allowed_chat_ids.contains(&chat_id);
                    if allowed {
                      if let Err(e) = tg_send_message(&client, &token, chat_id, "pong", Some(message_id)).await {
                        log::info!("telegram: send /ping reply failed: {e}");
                      }
                    } else {
                      if let Err(e) = tg_send_message(
                        &client,
                        &token,
                        chat_id,
                        "Нема доступу. Використай /whoami і додай chat_id в allowlist.",
                        Some(message_id),
                      )
                      .await
                      {
                        log::info!("telegram: send /ping deny failed: {e}");
                      }
                    }
                  }
                  Some("/codex") => {
                    let allowed = cfg.telegram.allowed_chat_ids.contains(&chat_id);
                    if !allowed {
                      if let Err(e) = tg_send_message(
                        &client,
                        &token,
                        chat_id,
                        "Нема доступу. Використай /whoami і додай chat_id в allowlist.",
                        Some(message_id),
                      )
                      .await
                      {
                        log::info!("telegram: send /codex deny failed: {e}");
                      }
                      continue;
                    }
                    let prompt = match rest {
                      Some(p) if !p.trim().is_empty() => p.trim().to_string(),
                      _ => {
                        if let Err(e) =
                          tg_send_message(&client, &token, chat_id, "Напиши: /codex <повідомлення>", Some(message_id))
                            .await
                        {
                          log::info!("telegram: send /codex help failed: {e}");
                        }
                        continue;
                      }
                    };

                    let client2 = client.clone();
                    let token2 = token.clone();
                    let codex = runtime.inner.codex.clone();
                    let logs2 = runtime.inner.logs.clone();
                    runtime.inner.logs.push(logbus::LogLevel::Info, "telegram", format!("codex request chat_id={chat_id}"));
                    tauri::async_runtime::spawn(async move {
                      let (typing_tx, mut typing_rx) = watch::channel(false);
                      // Standard Telegram loader while Codex works.
                      let client3 = client2.clone();
                      let token3 = token2.clone();
                      let logs3 = logs2.clone();
                      tauri::async_runtime::spawn(async move {
                        loop {
                          if *typing_rx.borrow() {
                            break;
                          }
                          if let Err(e) = tg_send_chat_action(&client3, &token3, chat_id, "typing").await {
                            logs3.push(logbus::LogLevel::Warn, "telegram", format!("sendChatAction failed: {e}"));
                            break;
                          }
                          tokio::select! {
                            _ = typing_rx.changed() => break,
                            _ = tokio::time::sleep(Duration::from_secs(4)) => {}
                          }
                        }
                      });

                      let mut stream = match codex.start_turn_stream(chat_id, &prompt).await {
                        Ok(s) => s,
                        Err(e) => {
                          let _ = typing_tx.send(true);
                          let msg = if e == "Busy" {
                            "Зачекай: обробляю попереднє повідомлення.".to_string()
                          } else {
                            format!("Codex error: {e}")
                          };
                          let _ = tg_send_message_series(&client2, &token2, chat_id, &msg, Some(message_id)).await;
                          return;
                        }
                      };

                      let mut first_reply = true;
                      let mut sent_any = false;
                      let mut done_rx = stream.done_rx;
                      loop {
                        tokio::select! {
                          maybe = stream.updates_rx.recv() => {
                            let Some(chunk) = maybe else { continue; };
                            let reply_to = if first_reply { Some(message_id) } else { None };
                            first_reply = false;
                            sent_any = true;
                            let _ = tg_send_message(&client2, &token2, chat_id, &chunk, reply_to).await;
                          }
                          done = &mut done_rx => {
                            // Stop typing loader.
                            let _ = typing_tx.send(true);
                            // Drain any chunks that were queued before completion.
                            while let Ok(chunk) = stream.updates_rx.try_recv() {
                              let reply_to = if first_reply { Some(message_id) } else { None };
                              first_reply = false;
                              sent_any = true;
                              let _ = tg_send_message(&client2, &token2, chat_id, &chunk, reply_to).await;
                            }

                            match done {
                              Ok(Ok(final_text)) => {
                                if !sent_any && !final_text.trim().is_empty() {
                                  let _ = tg_send_message_series(&client2, &token2, chat_id, &final_text, Some(message_id)).await;
                                }
                              }
                              Ok(Err(e)) => {
                                let msg = format!("Codex error: {e}");
                                let _ = tg_send_message_series(&client2, &token2, chat_id, &msg, Some(message_id)).await;
                              }
                              Err(_) => {
                                let _ = tg_send_message_series(&client2, &token2, chat_id, "Codex error: internal channel closed", Some(message_id)).await;
                              }
                            }
                            break;
                          }
                        }
                      }
                    });
                  }
                  _ => {}
                }

                // For allowlisted chats: treat any non-command message as Codex input.
                if cmd.is_none() {
                  let allowed = cfg.telegram.allowed_chat_ids.contains(&chat_id);
                  if allowed {
                    let prompt = trimmed.to_string();
                    let client2 = client.clone();
                    let token2 = token.clone();
                    let codex = runtime.inner.codex.clone();
                    let logs2 = runtime.inner.logs.clone();
                    runtime.inner.logs.push(logbus::LogLevel::Info, "telegram", format!("codex request chat_id={chat_id}"));
                    tauri::async_runtime::spawn(async move {
                      let (typing_tx, mut typing_rx) = watch::channel(false);
                      let client3 = client2.clone();
                      let token3 = token2.clone();
                      let logs3 = logs2.clone();
                      tauri::async_runtime::spawn(async move {
                        loop {
                          if *typing_rx.borrow() {
                            break;
                          }
                          if let Err(e) = tg_send_chat_action(&client3, &token3, chat_id, "typing").await {
                            logs3.push(logbus::LogLevel::Warn, "telegram", format!("sendChatAction failed: {e}"));
                            break;
                          }
                          tokio::select! {
                            _ = typing_rx.changed() => break,
                            _ = tokio::time::sleep(Duration::from_secs(4)) => {}
                          }
                        }
                      });

                      let mut stream = match codex.start_turn_stream(chat_id, &prompt).await {
                        Ok(s) => s,
                        Err(e) => {
                          let _ = typing_tx.send(true);
                          let msg = if e == "Busy" {
                            "Зачекай: обробляю попереднє повідомлення.".to_string()
                          } else {
                            format!("Codex error: {e}")
                          };
                          let _ = tg_send_message_series(&client2, &token2, chat_id, &msg, Some(message_id)).await;
                          return;
                        }
                      };

                      let mut first_reply = true;
                      let mut sent_any = false;
                      let mut done_rx = stream.done_rx;
                      loop {
                        tokio::select! {
                          maybe = stream.updates_rx.recv() => {
                            let Some(chunk) = maybe else { continue; };
                            let reply_to = if first_reply { Some(message_id) } else { None };
                            first_reply = false;
                            sent_any = true;
                            let _ = tg_send_message(&client2, &token2, chat_id, &chunk, reply_to).await;
                          }
                          done = &mut done_rx => {
                            let _ = typing_tx.send(true);
                            while let Ok(chunk) = stream.updates_rx.try_recv() {
                              let reply_to = if first_reply { Some(message_id) } else { None };
                              first_reply = false;
                              sent_any = true;
                              let _ = tg_send_message(&client2, &token2, chat_id, &chunk, reply_to).await;
                            }

                            match done {
                              Ok(Ok(final_text)) => {
                                if !sent_any && !final_text.trim().is_empty() {
                                  let _ = tg_send_message_series(&client2, &token2, chat_id, &final_text, Some(message_id)).await;
                                }
                              }
                              Ok(Err(e)) => {
                                let msg = format!("Codex error: {e}");
                                let _ = tg_send_message_series(&client2, &token2, chat_id, &msg, Some(message_id)).await;
                              }
                              Err(_) => {
                                let _ = tg_send_message_series(&client2, &token2, chat_id, "Codex error: internal channel closed", Some(message_id)).await;
                              }
                            }
                            break;
                          }
                        }
                      }
                    });
                  }
                }
              }
            }

            state.offset = new_offset;
            if let Err(e) = save_bot_state(&app, &state) {
              let mut st = runtime.inner.status.write().await;
              st.last_error = Some(e);
              runtime
                .inner
                .logs
                .push(logbus::LogLevel::Error, "telegram", "save bot state failed");
            }
          }
          Err(e) => {
            let mut st = runtime.inner.status.write().await;
            st.last_error = Some(e);
            runtime.inner.logs.push(logbus::LogLevel::Error, "telegram", format!("poll failed: {}", st.last_error.clone().unwrap_or_default()));
            // backoff a bit
            tokio::time::sleep(Duration::from_millis(800)).await;
          }
        }
      }

      // stopped
      *runtime.inner.stop_tx.write().await = None;
      let mut st = runtime.inner.status.write().await;
      st.running = false;
      runtime.inner.logs.push(logbus::LogLevel::Info, "telegram", "stopped");
    });

    Ok(())
  }

  pub async fn stop(&self) -> Result<(), String> {
    if let Some(tx) = self.inner.stop_tx.read().await.clone() {
      let _ = tx.send(true);
    }
    Ok(())
  }
}

fn now_unix_ms() -> u128 {
  use std::time::{SystemTime, UNIX_EPOCH};
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis()
}

#[derive(Debug, Deserialize)]
struct TgResponse<T> {
  ok: bool,
  result: Option<T>,
  description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TgUser {
  username: Option<String>,
}

pub(super) async fn tg_get_me(client: &Client, token: &str) -> Result<Option<String>, String> {
  let url = format!("https://api.telegram.org/bot{token}/getMe");
  let resp = client
    .get(url)
    .send()
    .await
    .map_err(|e| format!("getMe request failed: {e}"))?;
  let body: TgResponse<TgUser> = resp.json().await.map_err(|e| format!("getMe parse failed: {e}"))?;
  if !body.ok {
    return Err(body.description.unwrap_or_else(|| "getMe failed".to_string()));
  }
  Ok(body.result.and_then(|u| u.username))
}

#[derive(Debug, Deserialize)]
struct TgUpdate {
  update_id: i64,
  message: Option<TgMessage>,
}

#[derive(Debug, Deserialize)]
struct TgMessage {
  message_id: i64,
  chat: TgChat,
  text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TgChat {
  id: i64,
}

async fn tg_get_updates(
  client: &Client,
  token: &str,
  offset: i64,
  timeout_sec: i64,
) -> Result<(i64, Vec<TgUpdate>), String> {
  let url = format!("https://api.telegram.org/bot{token}/getUpdates");
  // Use query params for maximum compatibility with the Telegram Bot API.
  let resp = client
    .get(url)
    .query(&[("offset", offset), ("timeout", timeout_sec), ("limit", 50_i64)])
    .send()
    .await
    .map_err(|e| format!("getUpdates request failed: {e}"))?;
  let body: TgResponse<Vec<TgUpdate>> = resp
    .json()
    .await
    .map_err(|e| format!("getUpdates parse failed: {e}"))?;
  if !body.ok {
    return Err(body.description.unwrap_or_else(|| "getUpdates failed".to_string()));
  }
  let items = body.result.unwrap_or_default();
  let new_offset = items
    .iter()
    .map(|u| u.update_id + 1)
    .max()
    .unwrap_or(offset);
  Ok((new_offset, items))
}

pub(super) async fn tg_send_message(
  client: &Client,
  token: &str,
  chat_id: i64,
  text: &str,
  reply_to_message_id: Option<i64>,
) -> Result<(), String> {
  let url = format!("https://api.telegram.org/bot{token}/sendMessage");
  let mut text = text.to_string();
  // Telegram limit is 4096 chars; chunk when needed.
  while !text.is_empty() {
    let chunk: String = text.chars().take(4096).collect();
    text = text.chars().skip(4096).collect();
    let payload = serde_json::json!({
      "chat_id": chat_id,
      "text": chunk,
      "reply_to_message_id": reply_to_message_id
    });
    let resp = client
      .post(&url)
      .json(&payload)
      .send()
      .await
      .map_err(|e| format!("sendMessage request failed: {e}"))?;
    let body: TgResponse<serde_json::Value> = resp
      .json()
      .await
      .map_err(|e| format!("sendMessage parse failed: {e}"))?;
    if !body.ok {
      return Err(body.description.unwrap_or_else(|| "sendMessage failed".to_string()));
    }
  }
  Ok(())
}

async fn tg_send_chat_action(
  client: &Client,
  token: &str,
  chat_id: i64,
  action: &str,
) -> Result<(), String> {
  let url = format!("https://api.telegram.org/bot{token}/sendChatAction");
  let payload = serde_json::json!({
    "chat_id": chat_id,
    "action": action
  });
  let resp = client
    .post(&url)
    .json(&payload)
    .send()
    .await
    .map_err(|e| format!("sendChatAction request failed: {e}"))?;
  let body: TgResponse<serde_json::Value> = resp
    .json()
    .await
    .map_err(|e| format!("sendChatAction parse failed: {e}"))?;
  if !body.ok {
    return Err(body.description.unwrap_or_else(|| "sendChatAction failed".to_string()));
  }
  Ok(())
}

async fn tg_send_message_series(
  client: &Client,
  token: &str,
  chat_id: i64,
  text: &str,
  reply_to_message_id: Option<i64>,
) -> Result<(), String> {
  let parts = split_for_telegram(text, 900);
  let mut first = true;
  for part in parts {
    if part.trim().is_empty() {
      continue;
    }
    tg_send_message(
      client,
      token,
      chat_id,
      &part,
      if first { reply_to_message_id } else { None },
    )
    .await?;
    first = false;
    tokio::time::sleep(Duration::from_millis(220)).await;
  }
  Ok(())
}

fn split_for_telegram(text: &str, max_chars: usize) -> Vec<String> {
  let mut out: Vec<String> = vec![];
  let mut s = text.replace("\r\n", "\n");

  // Preserve paragraph breaks, but keep messages short (a few sentences).
  while !s.trim().is_empty() {
    // Find the byte index that corresponds to max_chars.
    let mut end_byte = s.len();
    let mut count = 0usize;
    for (i, _) in s.char_indices() {
      if count == max_chars {
        end_byte = i;
        break;
      }
      count += 1;
    }

    // Entire remainder fits in one message.
    if count <= max_chars {
      out.push(s.trim().to_string());
      break;
    }

    let window = &s[..end_byte]; // safe UTF-8 boundary from char_indices

    // Prefer to cut on a blank line within the window.
    if let Some(idx) = window.rfind("\n\n") {
      if idx > 0 {
        out.push(window[..idx].trim().to_string());
        s = s[idx + 2..].to_string();
        continue;
      }
    }

    // Try to end on a sentence boundary.
    let mut cut = None;
    for (i, ch) in window.char_indices().rev() {
      if ch == '.' || ch == '!' || ch == '?' || ch == '\n' {
        // Don't create tiny chunks.
        if i > 120 {
          cut = Some(i + ch.len_utf8());
          break;
        }
      }
    }
    let cut = cut.unwrap_or(end_byte);
    out.push(s[..cut].trim().to_string());
    s = s[cut..].to_string();
  }

  out
}

fn extract_text_message(update: &TgUpdate) -> Option<(i64, i64, String)> {
  let msg = update.message.as_ref()?;
  let text = msg.text.as_ref()?.to_string();
  Some((msg.chat.id, msg.message_id, text))
}

fn parse_command(text: &str) -> (Option<String>, Option<String>) {
  let first = text.split_whitespace().next().unwrap_or("");
  if !first.starts_with('/') {
    return (None, None);
  }
  let cmd = first.split('@').next().unwrap_or(first).to_string();
  let rest = text
    .strip_prefix(first)
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty());
  (Some(cmd), rest)
}

fn load_bot_state(app: &AppHandle) -> Result<BotState, String> {
  let path = paths::telegram_bot_state_path(app)?;
  if !path.exists() {
    return Ok(BotState::default());
  }
  let raw = fs::read_to_string(path).map_err(|e| format!("read bot state failed: {e}"))?;
  serde_json::from_str(&raw).map_err(|e| format!("parse bot state failed: {e}"))
}

fn save_bot_state(app: &AppHandle, state: &BotState) -> Result<(), String> {
  let path = paths::telegram_bot_state_path(app)?;
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|e| format!("create bot state dir failed: {e}"))?;
  }
  let raw = serde_json::to_string_pretty(state).map_err(|e| format!("serialize bot state failed: {e}"))?;
  fs::write(path, raw).map_err(|e| format!("write bot state failed: {e}"))
}
