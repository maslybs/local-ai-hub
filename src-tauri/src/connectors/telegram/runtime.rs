use std::{collections::HashMap, error::Error, fs, sync::Arc, time::Duration};

use reqwest::Client;
use serde::Deserialize;
use tauri::AppHandle;
use tokio::sync::{watch, RwLock};

use crate::core::{config_store::AppConfig, logbus, paths, secrets, time};
use crate::connectors::codex::runtime::CodexRuntime;

use super::types::{BotState, TelegramStatus};

const NO_ACCESS_MSG: &str = "Нема доступу. Використай /whoami і додай chat_id в allowlist.";

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
        // Long-polling can be flaky over HTTP/2 on some networks; Telegram fully supports HTTP/1.1.
        .http1_only()
        .build()
        .expect("reqwest client");

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

      // Cache of the most recent /threads results per chat, so user can select by number (/thread 3).
      let mut last_threads: HashMap<i64, Vec<String>> = HashMap::new();

      loop {
        if *rx.borrow() {
          break;
        }

        let cfg = runtime.inner.config.read().await.clone();
        let poll_timeout = cfg.telegram.poll_timeout_sec.clamp(1, 60) as i64;

        let updates_fut = tg_get_updates(&client, &token, state.offset, poll_timeout);
        let updates = tokio::select! {
          _ = rx.changed() => break,
          res = updates_fut => res,
        };

        match updates {
          Ok((new_offset, items)) => {
            {
              let mut st = runtime.inner.status.write().await;
              st.last_poll_unix_ms = Some(time::now_unix_ms());
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
                    } else if let Err(e) = tg_send_message(&client, &token, chat_id, NO_ACCESS_MSG, Some(message_id)).await {
                      log::info!("telegram: send /ping deny failed: {e}");
                    }
                  }
                  Some("/codex") => {
                    let allowed = cfg.telegram.allowed_chat_ids.contains(&chat_id);
                    if !allowed {
                      if let Err(e) = tg_send_message(&client, &token, chat_id, NO_ACCESS_MSG, Some(message_id))
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

                    runtime.inner.logs.push(logbus::LogLevel::Info, "telegram", format!("codex request chat_id={chat_id}"));
                    spawn_codex_reply(
                      client.clone(),
                      token.clone(),
                      runtime.inner.codex.clone(),
                      runtime.inner.logs.clone(),
                      chat_id,
                      message_id,
                      prompt,
                    );
                  }
                  Some("/threads") => {
                    let allowed = cfg.telegram.allowed_chat_ids.contains(&chat_id);
                    if !allowed {
                      if let Err(e) = tg_send_message(&client, &token, chat_id, NO_ACCESS_MSG, Some(message_id)).await {
                        log::info!("telegram: send /threads deny failed: {e}");
                      }
                      continue;
                    }

                    let res = runtime.inner.codex.list_threads(20, None).await;
                    let body = match res {
                      Ok(r) => {
                        if r.threads.is_empty() {
                          "Поки що немає діалогів.".to_string()
                        } else {
                          last_threads.insert(chat_id, r.threads.iter().take(10).map(|t| t.id.clone()).collect());
                          let mut out = String::new();
                          out.push_str("Останні діалоги:\n");
                          for (i, th) in r.threads.iter().take(10).enumerate() {
                            let title = th
                              .title
                              .clone()
                              .or_else(|| th.preview.clone())
                              .unwrap_or_else(|| "Діалог".to_string());
                            out.push_str(&format!("\n{}. {title}", i + 1));
                          }
                          out.push_str("\n\nПродовжити: /thread <номер> (наприклад /thread 1)");
                          out.trim().to_string()
                        }
                      }
                      Err(e) => format!("Codex error: {e}"),
                    };
                    if let Err(e) = tg_send_message_series(&client, &token, chat_id, &body, Some(message_id)).await {
                      log::info!("telegram: send /threads reply failed: {e}");
                    }
                  }
                  Some("/thread") => {
                    let allowed = cfg.telegram.allowed_chat_ids.contains(&chat_id);
                    if !allowed {
                      if let Err(e) = tg_send_message(&client, &token, chat_id, NO_ACCESS_MSG, Some(message_id)).await {
                        log::info!("telegram: send /thread deny failed: {e}");
                      }
                      continue;
                    }

                    let rest = rest.clone().unwrap_or_default();
                    if rest.trim().is_empty() {
                      let cur = runtime.inner.codex.get_chat_thread(chat_id).await;
                      let body = match cur {
                        Some(id) => format!("Поточний діалог:\n{id}\n\nЗмінити: /thread <id>\nСписок: /threads"),
                        None => "Немає вибраного діалогу.\n\nВибрати: /thread <id>\nСписок: /threads".to_string(),
                      };
                      if let Err(e) = tg_send_message_series(&client, &token, chat_id, &body, Some(message_id)).await {
                        log::info!("telegram: send /thread help failed: {e}");
                      }
                      continue;
                    }

                    let raw = rest.trim().to_string();
                    let thread_id = if let Ok(n) = raw.parse::<usize>() {
                      last_threads
                        .get(&chat_id)
                        .and_then(|v| v.get(n.saturating_sub(1)))
                        .cloned()
                        .unwrap_or_default()
                    } else {
                      raw
                    };
                    if thread_id.trim().is_empty() {
                      let msg = "Невірний номер. Спочатку виклич /threads і вибери 1-10.".to_string();
                      if let Err(e) = tg_send_message_series(&client, &token, chat_id, &msg, Some(message_id)).await {
                        log::info!("telegram: send /thread invalid failed: {e}");
                      }
                      continue;
                    }

                    // Attach this Telegram chat to a specific Codex thread id.
                    match runtime.inner.codex.attach_chat_to_thread(chat_id, thread_id).await {
                      Ok(_) => {
                        if let Err(e) = tg_send_message(&client, &token, chat_id, "OK. Підключив до вибраного діалогу.", Some(message_id)).await {
                          log::info!("telegram: send /thread ok failed: {e}");
                        }
                      }
                      Err(e) => {
                        let msg = format!("Codex error: {e}");
                        if let Err(e) = tg_send_message_series(&client, &token, chat_id, &msg, Some(message_id)).await {
                          log::info!("telegram: send /thread err failed: {e}");
                        }
                      }
                    }
                  }
                  _ => {}
                }

                // For allowlisted chats: treat any non-command message as Codex input.
                if cmd.is_none() {
                  let allowed = cfg.telegram.allowed_chat_ids.contains(&chat_id);
                  if allowed {
                    let prompt = trimmed.to_string();
                    runtime.inner.logs.push(logbus::LogLevel::Info, "telegram", format!("codex request chat_id={chat_id}"));
                    spawn_codex_reply(
                      client.clone(),
                      token.clone(),
                      runtime.inner.codex.clone(),
                      runtime.inner.logs.clone(),
                      chat_id,
                      message_id,
                      prompt,
                    );
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

fn spawn_codex_reply(
  client: Client,
  token: String,
  codex: CodexRuntime,
  logs: logbus::LogBus,
  chat_id: i64,
  message_id: i64,
  prompt: String,
) {
  tauri::async_runtime::spawn(async move {
    let (typing_tx, mut typing_rx) = watch::channel(false);

    // Standard Telegram loader while Codex works.
    let client3 = client.clone();
    let token3 = token.clone();
    let logs3 = logs.clone();
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
        if let Err(e) = tg_send_message_series(&client, &token, chat_id, &msg, Some(message_id)).await {
          logs.push(logbus::LogLevel::Warn, "telegram", format!("sendMessageSeries failed: {e}"));
        }
        return;
      }
    };

    logs.push(logbus::LogLevel::Info, "telegram", format!("codex stream started chat_id={chat_id}"));

    let mut first_reply = true;
    let mut sent_any = false;
    let mut updates_closed = false;
    let mut first_chunk_logged = false;
    let mut done_rx = stream.done_rx;
    loop {
      tokio::select! {
        maybe = stream.updates_rx.recv(), if !updates_closed => {
          let Some(chunk) = maybe else {
            updates_closed = true;
            logs.push(logbus::LogLevel::Warn, "telegram", format!("codex updates channel closed chat_id={chat_id}"));
            continue;
          };
          let reply_to = if first_reply { Some(message_id) } else { None };
          first_reply = false;
          sent_any = true;
          if !first_chunk_logged {
            first_chunk_logged = true;
            logs.push(logbus::LogLevel::Info, "telegram", format!("codex first chunk chat_id={chat_id}"));
          }
          if let Err(e) = tg_send_message(&client, &token, chat_id, &chunk, reply_to).await {
            logs.push(logbus::LogLevel::Warn, "telegram", format!("sendMessage failed: {e}"));
          }
        }
        done = &mut done_rx => {
          // Stop typing loader.
          let _ = typing_tx.send(true);
          // Drain any chunks that were queued before completion.
          while let Ok(chunk) = stream.updates_rx.try_recv() {
            let reply_to = if first_reply { Some(message_id) } else { None };
            first_reply = false;
            sent_any = true;
            if let Err(e) = tg_send_message(&client, &token, chat_id, &chunk, reply_to).await {
              logs.push(logbus::LogLevel::Warn, "telegram", format!("sendMessage failed: {e}"));
            }
          }

          match done {
            Ok(Ok(final_text)) => {
              logs.push(logbus::LogLevel::Info, "telegram", format!("codex done ok chat_id={chat_id} chars={}", final_text.chars().count()));
              if !sent_any {
                if final_text.trim().is_empty() {
                  let msg = "Нема відповіді від Codex. Спробуй ще раз.".to_string();
                  if let Err(e) = tg_send_message(&client, &token, chat_id, &msg, Some(message_id)).await {
                    logs.push(logbus::LogLevel::Warn, "telegram", format!("sendMessage failed: {e}"));
                  }
                } else if let Err(e) = tg_send_message_series(&client, &token, chat_id, &final_text, Some(message_id)).await {
                  logs.push(logbus::LogLevel::Warn, "telegram", format!("sendMessageSeries failed: {e}"));
                }
              }
            }
            Ok(Err(e)) => {
              logs.push(logbus::LogLevel::Warn, "telegram", format!("codex done err chat_id={chat_id}: {e}"));
              let msg = format!("Codex error: {e}");
              if let Err(e) = tg_send_message_series(&client, &token, chat_id, &msg, Some(message_id)).await {
                logs.push(logbus::LogLevel::Warn, "telegram", format!("sendMessageSeries failed: {e}"));
              }
            }
            Err(_) => {
              logs.push(logbus::LogLevel::Warn, "telegram", format!("codex done channel closed chat_id={chat_id}"));
              if let Err(e) = tg_send_message_series(&client, &token, chat_id, "Codex error: internal channel closed", Some(message_id)).await {
                logs.push(logbus::LogLevel::Warn, "telegram", format!("sendMessageSeries failed: {e}"));
              }
            }
          }
          break;
        }
      }
    }
  });
}

fn redact_token(s: &str, token: &str) -> String {
  if token.trim().is_empty() {
    return s.to_string();
  }
  s.replace(token, "[REDACTED]")
}

fn format_reqwest_error(e: &reqwest::Error, token: &str) -> String {
  let mut parts: Vec<String> = vec![];

  let mut base = e.to_string();
  base = redact_token(&base, token);
  parts.push(base);

  if e.is_timeout() {
    parts.push("timeout".to_string());
  }
  if e.is_connect() {
    parts.push("connect".to_string());
  }
  if let Some(status) = e.status() {
    parts.push(format!("http {status}"));
  }

  // Include a short error chain (often has the real cause).
  let mut src = e.source();
  let mut depth = 0usize;
  while let Some(s) = src {
    depth += 1;
    if depth > 4 {
      break;
    }
    let mut t = s.to_string();
    t = redact_token(&t, token);
    parts.push(t);
    src = s.source();
  }

  parts.join(": ")
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
    .map_err(|e| format!("getMe request failed: {}", format_reqwest_error(&e, token)))?;
  let status = resp.status();
  let raw = resp
    .text()
    .await
    .map_err(|e| format!("getMe read failed: {}", format_reqwest_error(&e, token)))?;
  let body: TgResponse<TgUser> = serde_json::from_str(&raw)
    .map_err(|e| format!("getMe parse failed (http {status}): {e}"))?;
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
    .map_err(|e| format!("getUpdates request failed: {}", format_reqwest_error(&e, token)))?;
  let status = resp.status();
  let raw = resp
    .text()
    .await
    .map_err(|e| format!("getUpdates read failed: {}", format_reqwest_error(&e, token)))?;
  let body: TgResponse<Vec<TgUpdate>> = serde_json::from_str(&raw)
    .map_err(|e| format!("getUpdates parse failed (http {status}): {e}"))?;
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
  let mut text = format_for_telegram(text);
  // Telegram limit is 4096 chars; chunk when needed.
  while !text.is_empty() {
    let chunk: String = text.chars().take(4096).collect();
    text = text.chars().skip(4096).collect();
    // Don't send `reply_to_message_id: null` (Telegram can reject nulls on some fields).
    let mut payload = serde_json::Map::new();
    payload.insert("chat_id".to_string(), serde_json::json!(chat_id));
    payload.insert("text".to_string(), serde_json::json!(chunk));
    if let Some(reply_to_message_id) = reply_to_message_id {
      payload.insert("reply_to_message_id".to_string(), serde_json::json!(reply_to_message_id));
    }
    payload.insert("disable_web_page_preview".to_string(), serde_json::json!(true));
    let resp = client
      .post(&url)
      .json(&payload)
      .send()
      .await
      .map_err(|e| format!("sendMessage request failed: {}", format_reqwest_error(&e, token)))?;
    let status = resp.status();
    let raw = resp
      .text()
      .await
      .map_err(|e| format!("sendMessage read failed: {}", format_reqwest_error(&e, token)))?;
    let body: TgResponse<serde_json::Value> = serde_json::from_str(&raw)
      .map_err(|e| format!("sendMessage parse failed (http {status}): {e}"))?;
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
    .map_err(|e| format!("sendChatAction request failed: {}", format_reqwest_error(&e, token)))?;
  let status = resp.status();
  let raw = resp
    .text()
    .await
    .map_err(|e| format!("sendChatAction read failed: {}", format_reqwest_error(&e, token)))?;
  let body: TgResponse<serde_json::Value> = serde_json::from_str(&raw)
    .map_err(|e| format!("sendChatAction parse failed (http {status}): {e}"))?;
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

    // Prefer cutting on a list boundary so we don't split bullets awkwardly.
    let list_markers = ["\n- ", "\n• ", "\n* ", "\n1. ", "\n2. ", "\n3. ", "\n4. "];
    let mut list_cut: Option<usize> = None;
    for m in list_markers {
      if let Some(idx) = window.rfind(m) {
        if idx > 120 {
          list_cut = Some(list_cut.map(|b| b.max(idx + 1)).unwrap_or(idx + 1));
        }
      }
    }

    // Try to end after a couple of completed sentences.
    let cut = list_cut.unwrap_or_else(|| {
      let mut ends: Vec<usize> = vec![];
      let mut chars = window.char_indices().peekable();
      while let Some((i, ch)) = chars.next() {
        if ch == '.' || ch == '!' || ch == '?' || ch == '…' {
          let next = chars.peek().map(|(_, c)| *c);
          if next.is_none() || next.map(|c| c.is_whitespace()).unwrap_or(false) {
            ends.push(i + ch.len_utf8());
          }
        }
      }
      // Use the 2nd sentence end when available (keeps "few sentences" feel).
      let desired = 2usize;
      if ends.len() >= desired && ends[desired - 1] > 80 {
        ends[desired - 1]
      } else {
        // Fallback: last whitespace.
        let mut last_ws = None;
        for (i, ch) in window.char_indices() {
          if ch.is_whitespace() && i > 120 {
            last_ws = Some(i);
          }
        }
        last_ws.unwrap_or(end_byte)
      }
    });
    out.push(s[..cut].trim().to_string());
    s = s[cut..].to_string();
  }

  out
}

fn format_for_telegram(input: &str) -> String {
  // Keep Telegram output readable:
  // - remove inline backticks (Telegram doesn't render them as code without parse_mode)
  // - drop noisy absolute paths in skill listings
  // - collapse excessive blank lines
  let mut s = input.replace("\r\n", "\n");

  // Special-case: "available skills" dumps look awful in Telegram due to long file paths.
  // Make it a clean bullet list.
  if s.contains("Доступні скіли") && (s.contains("SKILL.md") || s.contains("Файл:")) {
    if let Some(compact) = compact_skills_list(&s) {
      s = compact;
    }
  }

  // Generic cleanup.
  s = s.replace('`', "");

  // Remove "Файл: /abs/path" fragments to avoid giant wrapped lines.
  s = strip_file_paths(&s);

  collapse_blank_lines(&s, 2)
}

fn compact_skills_list(raw: &str) -> Option<String> {
  let raw = raw.replace("\r\n", "\n").replace('`', "");
  let mut items: Vec<(String, String)> = vec![];

  for line in raw.lines() {
    let mut l = line.trim();
    if l.is_empty() {
      continue;
    }
    if l.starts_with("Доступні скіли") {
      continue;
    }
    if l.starts_with('-') {
      l = l.trim_start_matches('-').trim();
    }

    // Drop file paths if present: keep only the description part.
    if let Some(idx) = l.find("Файл:") {
      l = l[..idx].trim();
    }

    // Parse "name — description"
    if let Some((name, desc)) = l.split_once('—') {
      let name = name.trim().to_string();
      let desc = name_desc_cleanup(desc);
      if !name.is_empty() && !desc.is_empty() {
        items.push((name, desc));
      }
      continue;
    }

    // Parse "name (description): /path/to/SKILL.md"
    if l.contains("SKILL.md") {
      let left = l.split(':').next().unwrap_or("").trim();
      if !left.is_empty() {
        // If we have parentheses, keep the part inside as description.
        if let (Some(a), Some(b)) = (left.find('('), left.rfind(')')) {
          if b > a {
            let name = left[..a].trim().to_string();
            let desc = left[a + 1..b].trim().to_string();
            if !name.is_empty() && !desc.is_empty() {
              items.push((name, desc));
            }
          }
        }
      }
    }
  }

  if items.is_empty() {
    return None;
  }

  let mut out = String::new();
  out.push_str("Доступні скіли:\n");
  for (name, desc) in items {
    let d = if desc.chars().count() > 160 {
      let short: String = desc.chars().take(157).collect();
      format!("{short}…")
    } else {
      desc
    };
    out.push_str(&format!("• {name} — {d}\n"));
  }
  Some(out.trim().to_string())
}

fn name_desc_cleanup(desc: &str) -> String {
  let mut d = desc.trim().to_string();
  // Normalize spacing that sometimes gets awkward wraps.
  d = d.replace("  ", " ");
  // If someone pasted "(file: ...)" keep it out for Telegram.
  if let Some(idx) = d.find("(file:") {
    d = d[..idx].trim().to_string();
  }
  d
}

fn strip_file_paths(s: &str) -> String {
  let mut out: Vec<String> = vec![];
  for line in s.lines() {
    let mut l = line.to_string();
    if let Some(idx) = l.find("Файл:") {
      l = l[..idx].trim_end().to_string();
    }
    // Replace common absolute home path prefix to reduce noise even if "Файл:" wasn't present.
    l = l.replace("/Users/", "~/");
    out.push(l);
  }
  out.join("\n").trim().to_string()
}

fn collapse_blank_lines(s: &str, max_run: usize) -> String {
  let mut out = String::with_capacity(s.len());
  let mut run = 0usize;
  for ch in s.chars() {
    if ch == '\n' {
      run += 1;
      if run <= max_run {
        out.push(ch);
      }
    } else {
      run = 0;
      out.push(ch);
    }
  }
  out.trim().to_string()
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
