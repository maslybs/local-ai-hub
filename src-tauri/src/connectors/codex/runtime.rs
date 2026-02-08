use std::{
  collections::{HashMap, HashSet},
  fs,
  process::Stdio,
  path::PathBuf,
  sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
  },
  time::Duration,
};

use serde_json::Value;
use tauri::AppHandle;
use tokio::{
  io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
  process::{Child, ChildStdin, Command},
  sync::{mpsc, oneshot, Mutex, RwLock},
};

use super::types::{CodexDoctor, CodexStatus};
use crate::core::{logbus, paths};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct CodexRuntime {
  inner: Arc<Inner>,
}

pub struct CodexStream {
  pub updates_rx: mpsc::UnboundedReceiver<String>,
  pub done_rx: oneshot::Receiver<Result<String, String>>,
}

struct Inner {
  status: RwLock<CodexStatus>,
  child: Mutex<Option<Child>>,
  stdin: Mutex<Option<BufWriter<ChildStdin>>>,
  lifecycle_lock: Mutex<()>,
  stderr_tail: Mutex<VecDeque<String>>,
  install_lock: Mutex<()>,
  next_id: AtomicU64,
  pending: Mutex<HashMap<u64, oneshot::Sender<Result<Value, String>>>>,
  pending_turns: Mutex<HashMap<String, PendingTurn>>,
  chat_threads: Mutex<HashMap<i64, String>>,
  resumed_threads: Mutex<HashSet<String>>,
  busy_chats: Mutex<HashSet<i64>>,
  chat_threads_path: Option<PathBuf>,
  codex_home_dir: Option<PathBuf>,
  codex_tools_dir: Option<PathBuf>,
  app_global_agents_override: Option<PathBuf>,
  logs: logbus::LogBus,
  default_cwd: RwLock<Option<String>>,
  universal_instructions: RwLock<String>,
  universal_fallback_only: RwLock<bool>,
}

struct PendingTurn {
  chat_id: i64,
  full_text: String,
  sent_byte: usize,
  updates_tx: mpsc::UnboundedSender<String>,
  done: oneshot::Sender<Result<String, String>>,
}

impl CodexRuntime {
  pub fn new(app: &AppHandle, logs: logbus::LogBus) -> Self {
    let (chat_threads_path, chat_threads) = load_chat_threads(app);
    let codex_home_dir = paths::codex_home_dir(app).ok();
    let codex_tools_dir = paths::codex_tools_dir(app).ok();
    let app_global_agents_override = paths::codex_global_agents_override_path(app).ok();
    let default_cwd = std::env::current_dir()
      .ok()
      .and_then(|p| p.to_str().map(|s| s.to_string()));
    Self {
      inner: Arc::new(Inner {
        status: RwLock::new(CodexStatus::default()),
        child: Mutex::new(None),
        stdin: Mutex::new(None),
        lifecycle_lock: Mutex::new(()),
        stderr_tail: Mutex::new(VecDeque::with_capacity(40)),
        install_lock: Mutex::new(()),
        next_id: AtomicU64::new(1),
        pending: Mutex::new(HashMap::new()),
        pending_turns: Mutex::new(HashMap::new()),
        chat_threads: Mutex::new(chat_threads),
        resumed_threads: Mutex::new(HashSet::new()),
        busy_chats: Mutex::new(HashSet::new()),
        chat_threads_path,
        codex_home_dir,
        codex_tools_dir,
        app_global_agents_override,
        logs,
        default_cwd: RwLock::new(default_cwd),
        universal_instructions: RwLock::new(String::new()),
        universal_fallback_only: RwLock::new(true),
      }),
    }
  }

  pub async fn set_workspace_dir(&self, workspace_dir: Option<String>) {
    let ws = workspace_dir.and_then(|s| {
      let t = s.trim().to_string();
      if t.is_empty() { None } else { Some(t) }
    });
    *self.inner.default_cwd.write().await = ws;
    // Best-effort: update override file if Codex is already running.
    self.prepare_codex_home().await;
  }

  pub async fn set_universal_instructions(&self, instructions: String, fallback_only: bool) {
    *self.inner.universal_instructions.write().await = instructions;
    *self.inner.universal_fallback_only.write().await = fallback_only;
    // Best-effort: update override file if Codex is already running.
    self.prepare_codex_home().await;
  }

  pub async fn status(&self) -> CodexStatus {
    self.inner.status.read().await.clone()
  }

  pub async fn reset_threads(&self) -> Result<(), String> {
    // Clear thread mappings so the next message starts a fresh Codex thread.
    {
      let mut resumed = self.inner.resumed_threads.lock().await;
      resumed.clear();
    }
    {
      let mut busy = self.inner.busy_chats.lock().await;
      busy.clear();
    }
    {
      let mut turns = self.inner.pending_turns.lock().await;
      turns.clear();
    }
    {
      let mut threads = self.inner.chat_threads.lock().await;
      threads.clear();
      persist_chat_threads(self.inner.chat_threads_path.as_ref(), &threads)?;
    }

    self
      .inner
      .logs
      .push(logbus::LogLevel::Warn, "codex", "reset all thread mappings");
    Ok(())
  }

  pub async fn stop(&self) -> Result<(), String> {
    let pid = {
      let child = self.inner.child.lock().await;
      child.as_ref().and_then(|c| c.id())
    };
    self
      .inner
      .logs
      .push(logbus::LogLevel::Warn, "codex", format!("stop() called pid={}", pid.map(|p| p.to_string()).unwrap_or_else(|| "none".to_string())));
    {
      let mut child = self.inner.child.lock().await;
      if let Some(c) = child.as_mut() {
        let _ = c.kill().await;
      }
      *child = None;
    }
    *self.inner.stdin.lock().await = None;
    {
      let mut st = self.inner.status.write().await;
      st.running = false;
      st.initialized = false;
    }
    self.inner.logs.push(logbus::LogLevel::Info, "codex", "stopped");
    Ok(())
  }

  pub async fn connect(&self) -> Result<(), String> {
    self.ensure_initialized().await
  }

  pub async fn doctor(&self) -> CodexDoctor {
    let local_entry = self.local_codex_entry_path();
    let local_codex_ok = local_entry.as_ref().map(|p| p.exists()).unwrap_or(false);
    let local_codex_version = self.local_codex_version();

    let (node_ok, npm_ok) = tokio::join!(
      check_cmd_version(node_cmd(), &["--version"]),
      check_cmd_version(npm_cmd(), &["--version"])
    );
    let node_ok = node_ok.is_some();
    let npm_ok = npm_ok.is_some();

    let codex_version = check_cmd_version(codex_cmd(), &["--version"]).await;
    let codex_ok = codex_version.is_some();

    CodexDoctor {
      node_ok,
      npm_ok,
      codex_ok,
      codex_version,
      local_codex_ok,
      local_codex_version,
      local_codex_entry: local_entry.and_then(|p| p.to_str().map(|s| s.to_string())),
    }
  }

  pub async fn install_local_codex(&self) -> Result<CodexDoctor, String> {
    let _guard = self.inner.install_lock.lock().await;

    let tools = self
      .inner
      .codex_tools_dir
      .clone()
      .ok_or_else(|| "codex tools dir unavailable".to_string())?;
    if let Err(e) = fs::create_dir_all(&tools) {
      return Err(format!("create codex tools dir failed: {e}"));
    }

    // Ensure prerequisites are present.
    if check_cmd_version(node_cmd(), &["--version"]).await.is_none() {
      return Err("Node.js is required to install/run Codex. Install Node.js first, then try again.".to_string());
    }
    if check_cmd_version(npm_cmd(), &["--version"]).await.is_none() {
      return Err("npm is required to install Codex. Install Node.js (includes npm) first, then try again.".to_string());
    }

    self
      .inner
      .logs
      .push(logbus::LogLevel::Info, "codex", "installing Codex (npm)...");

    // Install into app_data_dir/codex-tools so it doesn't require admin/sudo.
    let mut cmd = Command::new(npm_cmd());
    cmd
      .arg("install")
      .arg("--prefix")
      .arg(&tools)
      .arg("@openai/codex");

    let out = tokio::time::timeout(Duration::from_secs(12 * 60), cmd.output())
      .await
      .map_err(|_| "Codex install timed out".to_string())?
      .map_err(|e| format!("Failed to run npm: {e}"))?;

    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if !stdout.is_empty() {
      self.inner.logs.push(logbus::LogLevel::Info, "codex", format!("npm: {stdout}"));
    }
    if !stderr.is_empty() {
      self.inner.logs.push(logbus::LogLevel::Warn, "codex", format!("npm: {stderr}"));
    }
    if !out.status.success() {
      return Err(format!("Codex install failed (npm exit {}).", out.status.code().unwrap_or(-1)));
    }

    // Validate entry exists.
    let entry_ok = self.local_codex_entry_path().as_ref().map(|p| p.exists()).unwrap_or(false);
    if !entry_ok {
      return Err("Codex was installed but entry point was not found. Try again or reinstall.".to_string());
    }

    self
      .inner
      .logs
      .push(logbus::LogLevel::Info, "codex", "Codex installed");
    Ok(self.doctor().await)
  }

  pub async fn login_chatgpt(&self) -> Result<(String, String), String> {
    self.ensure_initialized().await?;
    self.inner.logs.push(logbus::LogLevel::Info, "codex", "login_chatgpt: start");
    let params = serde_json::json!({ "type": "chatgpt" });
    let res = self.send_request("account/login/start", params).await?;

    let auth_url = res
      .get("authUrl")
      .and_then(|v| v.as_str())
      .ok_or_else(|| "login response missing authUrl".to_string())?
      .to_string();
    let login_id = res
      .get("loginId")
      .and_then(|v| v.as_str())
      .ok_or_else(|| "login response missing loginId".to_string())?
      .to_string();

    {
      let mut st = self.inner.status.write().await;
      st.login_url = Some(auth_url.clone());
      st.login_id = Some(login_id.clone());
      st.last_error = None;
    }
    self
      .inner
      .logs
      .push(logbus::LogLevel::Info, "codex", "login_chatgpt: auth url received");

    Ok((auth_url, login_id))
  }

  pub async fn logout(&self) -> Result<(), String> {
    self.ensure_initialized().await?;
    let _ = self.send_request("account/logout", Value::Null).await?;
    let mut st = self.inner.status.write().await;
    st.auth_mode = None;
    st.login_url = None;
    st.login_id = None;
    Ok(())
  }

  async fn ensure_started_locked(&self) -> Result<(), String> {
    if self.inner.status.read().await.running {
      return Ok(());
    }

    self.prepare_codex_home().await;

    self.inner.logs.push(logbus::LogLevel::Info, "codex", "starting app-server");
    let mut child = self.spawn_codex_app_server().await?;
    let pid = child.id();
    let stdin = child.stdin.take().ok_or_else(|| "Failed to open codex stdin".to_string())?;
    let stdout = child.stdout.take().ok_or_else(|| "Failed to open codex stdout".to_string())?;
    let stderr = child.stderr.take().ok_or_else(|| "Failed to open codex stderr".to_string())?;

    *self.inner.stdin.lock().await = Some(BufWriter::new(stdin));
    *self.inner.child.lock().await = Some(child);
    {
      let mut st = self.inner.status.write().await;
      st.running = true;
      st.initialized = false;
      st.last_error = None;
    }
    self
      .inner
      .logs
      .push(logbus::LogLevel::Info, "codex", format!("app-server running pid={}", pid.map(|p| p.to_string()).unwrap_or_else(|| "?".to_string())));

    let inner = self.inner.clone();
    tauri::async_runtime::spawn(async move {
      let mut lines = BufReader::new(stdout).lines();
      while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim();
        if line.is_empty() {
          continue;
        }
        match serde_json::from_str::<Value>(line) {
          Ok(msg) => handle_server_msg(inner.clone(), msg).await,
          Err(e) => {
            // Be tolerant: if codex prints a non-JSON warning to stdout, don't brick the connection.
            // Log it and keep reading.
            let snippet: String = line.chars().take(220).collect();
            inner.logs.push(
              logbus::LogLevel::Warn,
              "codex",
              format!("codex stdout non-JSON line (ignored): {e}: {snippet}"),
            );
          }
        }
      }

      // The app-server closed stdout (process exited or stdio was closed).
      // Fail any pending requests/turns so callers (Telegram/UI) don't hang until timeouts.
      {
        let mut pending = inner.pending.lock().await;
        for (_, tx) in pending.drain() {
          let _ = tx.send(Err("codex app-server stopped".to_string()));
        }
      }
      {
        let mut turns = inner.pending_turns.lock().await;
        let items: Vec<(String, PendingTurn)> = turns.drain().collect();
        drop(turns);
        for (_, p) in items {
          {
            let mut busy = inner.busy_chats.lock().await;
            busy.remove(&p.chat_id);
          }
          let _ = p.done.send(Err("Codex disconnected".to_string()));
        }
      }

      let mut st = inner.status.write().await;
      st.running = false;
      st.initialized = false;
      if st.last_error.is_none() {
        st.last_error = Some("codex app-server stopped".to_string());
      }
      inner.logs.push(logbus::LogLevel::Warn, "codex", "app-server stopped");

      // If stdout is closed, the stdio RPC transport is broken. Kill the child (if any) so a
      // subsequent connect can restart cleanly.
      {
        let mut child = inner.child.lock().await;
        if let Some(c) = child.as_mut() {
          let _ = c.kill().await;
        }
        *child = None;
      }
      *inner.stdin.lock().await = None;
    });

    let inner2 = self.inner.clone();
    tauri::async_runtime::spawn(async move {
      let mut lines = BufReader::new(stderr).lines();
      while let Ok(Some(line)) = lines.next_line().await {
        log::info!("codex(app-server): {}", line);
        handle_codex_stderr(inner2.clone(), line).await;
      }
    });

    // Monitor for child exit and log exit status + stderr tail.
    let inner3 = self.inner.clone();
    tauri::async_runtime::spawn(async move {
      loop {
        tokio::time::sleep(Duration::from_millis(350)).await;

        let status_opt = {
          let mut child = inner3.child.lock().await;
          let Some(c) = child.as_mut() else { return; };
          match c.try_wait() {
            Ok(s) => s,
            Err(e) => {
              inner3.logs.push(logbus::LogLevel::Warn, "codex", format!("try_wait failed: {e}"));
              None
            }
          }
        };

        if let Some(status) = status_opt {
          let code = status.code().map(|c| c.to_string()).unwrap_or_else(|| "none".to_string());
          #[cfg(unix)]
          let sig = {
            use std::os::unix::process::ExitStatusExt;
            status.signal().map(|s| s.to_string()).unwrap_or_else(|| "none".to_string())
          };
          #[cfg(not(unix))]
          let sig = "n/a".to_string();

          let tail = {
            let t = inner3.stderr_tail.lock().await;
            t.iter().cloned().collect::<Vec<_>>().join("\n")
          };

          inner3.logs.push(
            logbus::LogLevel::Error,
            "codex",
            format!("app-server exited code={code} signal={sig} pid={}", pid.map(|p| p.to_string()).unwrap_or_else(|| "?".to_string())),
          );
          if !tail.trim().is_empty() {
            inner3.logs.push(logbus::LogLevel::Error, "codex", format!("app-server stderr tail:\n{tail}"));
          }

          // Clean up handles so future calls can restart.
          *inner3.stdin.lock().await = None;
          *inner3.child.lock().await = None;
          return;
        }
      }
    });

    Ok(())
  }

  pub async fn ensure_initialized(&self) -> Result<(), String> {
    let _guard = self.inner.lifecycle_lock.lock().await;
    self.ensure_started_locked().await?;
    if self.inner.status.read().await.initialized {
      return Ok(());
    }

    self.inner.logs.push(logbus::LogLevel::Info, "codex", "initialize");
    let params = serde_json::json!({
      "clientInfo": {
        "name": "local-ai-hub",
        "title": "Local AI Hub",
        "version": env!("CARGO_PKG_VERSION"),
      }
    });
    let _ = self.send_request("initialize", params).await?;
    self.send_notification("initialized", None).await?;

    {
      let mut st = self.inner.status.write().await;
      st.initialized = true;
    }
    self.inner.logs.push(logbus::LogLevel::Info, "codex", "initialized");

    // Populate auth status early so UI/Telegram can surface "sign in required" without waiting for failures.
    let _ = self.refresh_account_state().await;
    Ok(())
  }

  pub async fn start_turn_stream(&self, chat_id: i64, text: &str) -> Result<CodexStream, String> {
    let text = text.trim();
    if text.is_empty() {
      return Err("Empty message".to_string());
    }

    {
      let mut busy = self.inner.busy_chats.lock().await;
      if busy.contains(&chat_id) {
        return Err("Busy".to_string());
      }
      busy.insert(chat_id);
    }

    // If anything fails before we register the turn, make sure we clear the busy flag.
    let started = self.start_turn_stream_inner(chat_id, text).await;
    if started.is_err() {
      let mut busy = self.inner.busy_chats.lock().await;
      busy.remove(&chat_id);
    }
    started
  }

  async fn start_turn_stream_inner(&self, chat_id: i64, text: &str) -> Result<CodexStream, String> {
    self.ensure_initialized().await?;
    self.ensure_account_ready().await?;

    let thread_id = self.thread_for_chat(chat_id).await?;
    self
      .inner
      .logs
      .push(logbus::LogLevel::Info, "codex", format!("turn/start chat_id={chat_id}"));

    let mut params = serde_json::json!({
      "threadId": thread_id,
      "approvalPolicy": "never",
      "input": [
        { "type": "text", "text": text }
      ]
    });
    if let Some(cwd) = self.inner.default_cwd.read().await.clone() {
      params["cwd"] = Value::String(cwd);
    }
    let turn_start = match self.send_request("turn/start", params.clone()).await {
      Ok(v) => v,
      Err(e) => {
        if let Some(bad) = extract_no_rollout_thread_id(&e) {
          self
            .inner
            .logs
            .push(logbus::LogLevel::Warn, "codex", "turn/start failed (no rollout); resetting thread and retrying");
          // Reset mapping(s) and retry with a fresh thread.
          // Use a full reset to handle any persistence/migration mismatches robustly.
          let _ = self.reset_threads().await;
          self.reset_thread_everywhere(&bad).await;
          let new_thread_id = self.thread_for_chat(chat_id).await?;
          params["threadId"] = Value::String(new_thread_id);
          self.send_request("turn/start", params).await?
        } else {
          return Err(e);
        }
      }
    };
    let turn_id = turn_start
      .get("turn")
      .and_then(|t| t.get("id"))
      .and_then(|id| id.as_str())
      .ok_or_else(|| "turn/start response missing turn.id".to_string())?
      .to_string();

    let (updates_tx, updates_rx) = mpsc::unbounded_channel::<String>();
    let (done_tx, done_rx) = oneshot::channel::<Result<String, String>>();

    {
      let mut turns = self.inner.pending_turns.lock().await;
      turns.insert(
        turn_id.clone(),
        PendingTurn {
          chat_id,
          full_text: String::new(),
          sent_byte: 0,
          updates_tx,
          done: done_tx,
        },
      );
    }

    // Safety timeout: if we never get turn/completed, fail the turn so callers can stop waiting.
    let inner = self.inner.clone();
    tauri::async_runtime::spawn(async move {
      tokio::time::sleep(Duration::from_secs(180)).await;
      let mut turns = inner.pending_turns.lock().await;
      if let Some(p) = turns.remove(&turn_id) {
        {
          let mut busy = inner.busy_chats.lock().await;
          busy.remove(&p.chat_id);
        }
        let _ = p.done.send(Err("Codex timeout".to_string()));
      }
    });

    Ok(CodexStream { updates_rx, done_rx })
  }

  async fn ensure_account_ready(&self) -> Result<(), String> {
    // If Codex requires OpenAI auth and we have no account, fail fast with a user-friendly error.
    let (requires, have) = self.refresh_account_state().await.unwrap_or((false, false));
    if requires && !have {
      return Err("Codex requires sign-in. Open AI Core -> Codex settings and sign in (ChatGPT).".to_string());
    }
    Ok(())
  }

  async fn refresh_account_state(&self) -> Result<(bool, bool), String> {
    // Docs: account/read -> { requiresOpenaiAuth, account }
    let res = self
      .send_request("account/read", serde_json::json!({ "refreshToken": false }))
      .await?;
    let requires = res
      .get("requiresOpenaiAuth")
      .and_then(|v| v.as_bool())
      .unwrap_or(false);
    let account = res.get("account").cloned().unwrap_or(Value::Null);

    let auth_mode = account
      .get("type")
      .and_then(|v| v.as_str())
      .map(|s| s.to_string());

    let have = auth_mode.is_some();

    {
      let mut st = self.inner.status.write().await;
      st.auth_mode = auth_mode;
      // Don't overwrite explicit runtime errors unless this is a clear auth requirement.
      if requires && !have {
        st.last_error = Some("Sign in required".to_string());
      } else if st.last_error.as_deref() == Some("Sign in required") {
        st.last_error = None;
      }
    }

    Ok((requires, have))
  }

  async fn thread_for_chat(&self, chat_id: i64) -> Result<String, String> {
    if let Some(t) = self.inner.chat_threads.lock().await.get(&chat_id).cloned() {
      match self.resume_thread_if_needed(&t).await {
        Ok(_) => return Ok(t),
        Err(e) => {
          if let Some(bad) = extract_no_rollout_thread_id(&e) {
            self
              .inner
              .logs
              .push(logbus::LogLevel::Warn, "codex", "thread invalid (no rollout); resetting");
            // Reset mapping(s) so the next call creates a fresh thread.
            // Use a full reset to handle any persistence/migration mismatches robustly.
            let _ = self.reset_threads().await;
            // Also best-effort clear any other references to the thread id mentioned in the error.
            self.reset_thread_everywhere(&bad).await;
            // Fall through and create a new thread.
          } else {
            return Err(e);
          }
        }
      }
    }

    self
      .inner
      .logs
      .push(logbus::LogLevel::Info, "codex", format!("thread/start chat_id={chat_id}"));
    let mut params = serde_json::json!({
      "approvalPolicy": "never",
      "sandbox": "read-only"
    });
    if let Some(cwd) = self.inner.default_cwd.read().await.clone() {
      params["cwd"] = Value::String(cwd);
    }
    let res = self.send_request("thread/start", params).await?;
    let thread_id = res
      .get("thread")
      .and_then(|t| t.get("id"))
      .and_then(|id| id.as_str())
      .ok_or_else(|| "thread/start response missing thread.id".to_string())?
      .to_string();

    {
      let mut guard = self.inner.chat_threads.lock().await;
      guard.insert(chat_id, thread_id.clone());
      persist_chat_threads(self.inner.chat_threads_path.as_ref(), &guard)?;
    }
    Ok(thread_id)
  }

  async fn resume_thread_if_needed(&self, thread_id: &str) -> Result<(), String> {
    let thread_id = thread_id.to_string();
    {
      let resumed = self.inner.resumed_threads.lock().await;
      if resumed.contains(&thread_id) {
        return Ok(());
      }
    }

    self
      .inner
      .logs
      .push(logbus::LogLevel::Info, "codex", format!("thread/resume thread_id={}", thread_id));
    // Resume is required after server restart to load the stored thread context.
    let mut params = serde_json::json!({
      "threadId": thread_id,
      "approvalPolicy": "never",
      "sandbox": "read-only"
    });
    if let Some(cwd) = self.inner.default_cwd.read().await.clone() {
      params["cwd"] = Value::String(cwd);
    }
    let _ = self.send_request("thread/resume", params).await?;

    let mut resumed = self.inner.resumed_threads.lock().await;
    resumed.insert(thread_id);
    Ok(())
  }

  async fn send_notification(&self, method: &str, params: Option<Value>) -> Result<(), String> {
    let msg = if let Some(params) = params {
      serde_json::json!({ "method": method, "params": params })
    } else {
      serde_json::json!({ "method": method })
    };
    self.write_msg(&msg).await
  }

  async fn send_request(&self, method: &str, params: Value) -> Result<Value, String> {
    let id = self.inner.next_id.fetch_add(1, Ordering::SeqCst);
    let msg = serde_json::json!({
      "id": id,
      "method": method,
      "params": params
    });

    let (tx, rx) = oneshot::channel::<Result<Value, String>>();
    self.inner.pending.lock().await.insert(id, tx);
    self.write_msg(&msg).await?;

    tokio::time::timeout(Duration::from_secs(60), rx)
      .await
      .map_err(|_| format!("codex request timeout: {method}"))?
      .map_err(|_| format!("codex request cancelled: {method}"))?
  }

  async fn write_msg(&self, msg: &Value) -> Result<(), String> {
    let raw = serde_json::to_string(msg).map_err(|e| format!("serialize failed: {e}"))?;
    let mut stdin = self.inner.stdin.lock().await;
    let w = stdin.as_mut().ok_or_else(|| "codex app-server not running".to_string())?;
    w.write_all(raw.as_bytes())
      .await
      .map_err(|e| format!("write failed: {e}"))?;
    w.write_all(b"\n")
      .await
      .map_err(|e| format!("write newline failed: {e}"))?;
    w.flush().await.map_err(|e| format!("flush failed: {e}"))?;
    Ok(())
  }
}

impl CodexRuntime {
  async fn spawn_codex_app_server(&self) -> Result<Child, String> {
    let env_home = self
      .inner
      .codex_home_dir
      .clone()
      .and_then(|p| p.to_str().map(|s| s.to_string()));

    // Preferred: run the app-local Codex install (no global dependency).
    if let Some(entry) = self.local_codex_entry_path().filter(|p| p.exists()) {
      let mut cmd = Command::new(node_cmd());
      cmd
        .arg(entry)
        .arg("app-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
      if let Some(home) = env_home.clone() {
        cmd.env("CODEX_HOME", home);
      }
      return match cmd.spawn() {
        Ok(c) => Ok(c),
        Err(e) => self
          .fail_start(format!("Failed to start codex app-server (local install): {e}"))
          .await,
      };
    }

    // No fallback: we want a deterministic, app-managed install for reliability (and Windows parity).
    self
      .fail_start("Codex is not installed. Install it in Codex settings.".to_string())
      .await
  }

  async fn fail_start<T>(&self, msg: String) -> Result<T, String> {
    {
      let mut st = self.inner.status.write().await;
      st.running = false;
      st.initialized = false;
      st.last_error = Some(msg.clone());
    }
    self.inner.logs.push(logbus::LogLevel::Error, "codex", msg.clone());
    Err(msg)
  }

  async fn prepare_codex_home(&self) {
    let Some(home) = self.inner.codex_home_dir.clone() else { return; };
    let _ = fs::create_dir_all(&home);

    // If the user is already logged into Codex (e.g. via Codex Desktop), import auth so this app
    // works "immediately" without forcing a second login flow.
    self.import_user_codex_auth(&home).await;

    // Best-effort: reuse user's installed skills by linking ~/.codex/skills into our app profile.
    // This avoids surprising "no skills" behavior when we isolate CODEX_HOME.
    #[cfg(unix)]
    {
      use std::os::unix::fs as unix_fs;
      let skills_dst = home.join("skills");
      if !skills_dst.exists() {
        if let Some(home_dir) = std::env::var_os("HOME") {
          let skills_src = PathBuf::from(home_dir).join(".codex").join("skills");
          if skills_src.exists() {
            let _ = unix_fs::symlink(&skills_src, &skills_dst);
          }
        }
      }
    }

    // Profile migration guard:
    // We run Codex app-server with an app-specific CODEX_HOME (state db lives there).
    // Old Telegram chat -> thread mappings created under a different CODEX_HOME (e.g. default ~/.codex)
    // are invalid in this profile and cause "no rollout found" errors. Clear them once.
    let marker = home.join(".local-ai-hub-profile-v1");
    if !marker.exists() {
      let _ = fs::write(&marker, "v1\n");
      #[cfg(unix)]
      {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&marker, fs::Permissions::from_mode(0o600));
      }

      {
        let mut resumed = self.inner.resumed_threads.lock().await;
        resumed.clear();
      }
      {
        let mut threads = self.inner.chat_threads.lock().await;
        if !threads.is_empty() {
          threads.clear();
          let _ = persist_chat_threads(self.inner.chat_threads_path.as_ref(), &threads);
        }
      }

      self.inner.logs.push(
        logbus::LogLevel::Warn,
        "codex",
        "initialized CODEX_HOME profile; cleared old chat thread mappings".to_string(),
      );
    }

    // Write/clear the app-global AGENTS.override.md based on config and workspace contents.
    let Some(override_path) = self.inner.app_global_agents_override.clone() else { return; };
    let instructions = self.inner.universal_instructions.read().await.clone();
    let fallback_only = *self.inner.universal_fallback_only.read().await;
    let ws = self.inner.default_cwd.read().await.clone();

    let should_apply = if instructions.trim().is_empty() {
      false
    } else if !fallback_only {
      true
    } else {
      // Apply only when the workspace has no AGENTS files.
      match ws {
        None => true,
        Some(dir) => {
          let p = PathBuf::from(dir);
          !(p.join("AGENTS.md").exists() || p.join("AGENTS.override.md").exists())
        }
      }
    };

    let content = if should_apply { instructions } else { String::new() };
    if let Some(parent) = override_path.parent() {
      let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&override_path, content);
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let _ = fs::set_permissions(&override_path, fs::Permissions::from_mode(0o600));
    }

    if should_apply {
      self.inner.logs.push(logbus::LogLevel::Info, "codex", "using universal instructions (AGENTS.override.md)");
    }
  }

  async fn import_user_codex_auth(&self, codex_home: &std::path::Path) {
    // Import auth/config from ~/.codex into our app profile if missing.
    // This keeps state isolated while preserving "already signed in" behavior.
    let Some(home_dir) = std::env::var_os("HOME") else { return; };
    let user_home = PathBuf::from(home_dir).join(".codex");

    let auth_src = user_home.join("auth.json");
    let auth_dst = codex_home.join("auth.json");
    if !auth_dst.exists() && auth_src.exists() {
      match fs::copy(&auth_src, &auth_dst) {
        Ok(_) => {
          #[cfg(unix)]
          {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&auth_dst, fs::Permissions::from_mode(0o600));
          }
          self.inner.logs.push(logbus::LogLevel::Info, "codex", "imported auth.json from ~/.codex");
        }
        Err(e) => {
          self
            .inner
            .logs
            .push(logbus::LogLevel::Warn, "codex", format!("failed to import auth.json: {e}"));
        }
      }
    }

    // Optional: reuse the user's Codex config defaults (model, features, etc.) if we have none yet.
    let cfg_src = user_home.join("config.toml");
    let cfg_dst = codex_home.join("config.toml");
    if !cfg_dst.exists() && cfg_src.exists() && fs::copy(&cfg_src, &cfg_dst).is_ok() {
      #[cfg(unix)]
      {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&cfg_dst, fs::Permissions::from_mode(0o600));
      }
      self.inner.logs.push(logbus::LogLevel::Info, "codex", "imported config.toml from ~/.codex");
    }
  }
}

impl CodexRuntime {
  fn local_codex_entry_path(&self) -> Option<PathBuf> {
    let tools = self.inner.codex_tools_dir.clone()?;
    Some(
      tools
        .join("node_modules")
        .join("@openai")
        .join("codex")
        .join("bin")
        .join("codex.js"),
    )
  }

  fn local_codex_version(&self) -> Option<String> {
    let tools = self.inner.codex_tools_dir.clone()?;
    let pkg = tools
      .join("node_modules")
      .join("@openai")
      .join("codex")
      .join("package.json");
    let raw = fs::read_to_string(pkg).ok()?;
    let v: Value = serde_json::from_str(&raw).ok()?;
    v.get("version").and_then(|x| x.as_str()).map(|s| s.to_string())
  }
}

#[cfg(windows)]
fn npm_cmd() -> &'static str {
  "npm.cmd"
}
#[cfg(not(windows))]
fn npm_cmd() -> &'static str {
  "npm"
}

fn node_cmd() -> &'static str {
  "node"
}

async fn check_cmd_version(cmd: &str, args: &[&str]) -> Option<String> {
  let mut c = Command::new(cmd);
  c.args(args);
  let out = match tokio::time::timeout(Duration::from_secs(8), c.output()).await {
    Ok(Ok(o)) => o,
    _ => return None,
  };
  if !out.status.success() {
    return None;
  }
  let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
  if s.is_empty() {
    return None;
  }
  Some(s)
}

#[cfg(windows)]
fn codex_cmd() -> &'static str {
  "codex.cmd"
}
#[cfg(not(windows))]
fn codex_cmd() -> &'static str {
  "codex"
}

impl CodexRuntime {
  async fn reset_thread_everywhere(&self, thread_id: &str) {
    let thread_id = thread_id.to_string();

    {
      let mut resumed = self.inner.resumed_threads.lock().await;
      resumed.remove(&thread_id);
    }

    let mut removed_any = false;
    {
      let mut threads = self.inner.chat_threads.lock().await;
      let keys: Vec<i64> = threads
        .iter()
        .filter_map(|(k, v)| if v == &thread_id { Some(*k) } else { None })
        .collect();
      for k in keys {
        threads.remove(&k);
        removed_any = true;
      }
      if removed_any {
        let _ = persist_chat_threads(self.inner.chat_threads_path.as_ref(), &threads);
      }
    }

    if removed_any {
      self
        .inner
        .logs
        .push(logbus::LogLevel::Warn, "codex", "reset stored thread mapping");
    }
  }
}

async fn handle_server_msg(inner: Arc<Inner>, msg: Value) {
  // Response: { id, result } or { id, error }
  if msg.get("id").is_some() && (msg.get("result").is_some() || msg.get("error").is_some()) {
    if let Some(id) = msg.get("id").and_then(|v| v.as_u64()) {
      if let Some(tx) = inner.pending.lock().await.remove(&id) {
        if let Some(err) = msg.get("error") {
          let _ = tx.send(Err(format!("{err}")));
        } else {
          let _ = tx.send(Ok(msg.get("result").cloned().unwrap_or(Value::Null)));
        }
      }
    }
    return;
  }

  // Server request: { id, method, params } - we auto-decline approvals for now.
  if msg.get("id").is_some() && msg.get("method").is_some() && msg.get("result").is_none() {
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");

    let result = match method {
      "item/commandExecution/requestApproval" => Some(serde_json::json!({ "decision": "decline" })),
      "item/fileChange/requestApproval" => Some(serde_json::json!({ "decision": "decline" })),
      "item/tool/requestUserInput" => Some(serde_json::json!({ "answers": {} })),
      _ => None,
    };

    if let Some(result) = result {
      let resp = serde_json::json!({ "id": id, "result": result });
      let mut stdin = inner.stdin.lock().await;
      if let Some(w) = stdin.as_mut() {
        if let Ok(raw) = serde_json::to_string(&resp) {
          let _ = w.write_all(raw.as_bytes()).await;
          let _ = w.write_all(b"\n").await;
          let _ = w.flush().await;
        }
      }
    }
    return;
  }

  // Notification: { method, params? }
  let method = match msg.get("method").and_then(|m| m.as_str()) {
    Some(m) => m,
    None => return,
  };
  let params = msg.get("params").cloned().unwrap_or(Value::Null);

  match method {
    "account/updated" => {
      let auth_mode = params.get("authMode").and_then(|v| v.as_str()).map(|s| s.to_string());
      let mut st = inner.status.write().await;
      st.auth_mode = auth_mode;
    }
    "account/login/completed" => {
      let success = params.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
      let login_id = params.get("loginId").and_then(|v| v.as_str()).map(|s| s.to_string());
      let err = params.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());

      let mut st = inner.status.write().await;
      if let Some(id) = login_id {
        if st.login_id.as_deref() == Some(id.as_str()) {
          st.login_url = None;
          st.login_id = None;
        }
      }
      if !success {
        st.last_error = Some(err.unwrap_or_else(|| "Login failed".to_string()));
      }
    }
    // Codex protocol has evolved; treat any item/*/delta that contains { turnId, delta } as assistant text.
    m if m.starts_with("item/") && m.ends_with("/delta") => {
      let turn_id = params.get("turnId").and_then(|v| v.as_str()).unwrap_or("").to_string();
      if turn_id.is_empty() {
        return;
      }
      let delta = params.get("delta").and_then(|v| v.as_str()).unwrap_or("");
      if delta.is_empty() {
        return;
      }
      let mut turns = inner.pending_turns.lock().await;
      if let Some(p) = turns.get_mut(&turn_id) {
        p.full_text.push_str(delta);
        flush_turn_chunks(p, false);
      }
    }
    "item/completed" => {
      let turn_id = params.get("turnId").and_then(|v| v.as_str()).unwrap_or("").to_string();
      if turn_id.is_empty() {
        return;
      }
      let item = params.get("item").cloned().unwrap_or(Value::Null);
      let ty = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
      let is_message = ty == "agentMessage" || ty == "assistantMessage" || ty.ends_with("Message");
      if !is_message {
        return;
      }
      if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
        let mut turns = inner.pending_turns.lock().await;
        if let Some(p) = turns.get_mut(&turn_id) {
          // item/completed contains the full text for this message.
          // Prefer it over deltas (it can be more complete).
          if text.len() >= p.full_text.len() {
            p.full_text = text.to_string();
          }
          flush_turn_chunks(p, false);
        }
      }
    }
    "turn/completed" => {
      let turn = params.get("turn").cloned().unwrap_or(Value::Null);
      let turn_id = turn.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
      if turn_id.is_empty() {
        return;
      }

      let status = turn.get("status").and_then(|v| v.as_str()).unwrap_or("");
      let err_msg = turn
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .map(|s| s.to_string());

      let mut turns = inner.pending_turns.lock().await;
      if let Some(mut p) = turns.remove(&turn_id) {
        flush_turn_chunks(&mut p, true);
        // clear per-chat busy state even if we fail to send back on the channel
        {
          let mut busy = inner.busy_chats.lock().await;
          busy.remove(&p.chat_id);
        }

        if status == "failed" {
          let _ = p.done.send(Err(err_msg.unwrap_or_else(|| "Turn failed".to_string())));
        } else {
          let _ = p.done.send(Ok(p.full_text));
        }
      }
    }
    "error" => {
      let mut st = inner.status.write().await;
      st.last_error = Some(format!("codex error: {params}"));
    }
    _ => {}
  }
}

async fn handle_codex_stderr(inner: Arc<Inner>, line: String) {
  // Keep a short tail for post-mortem when the process exits without a visible error.
  {
    let mut t = inner.stderr_tail.lock().await;
    if t.len() >= 30 {
      t.pop_front();
    }
    t.push_back(line.clone());
  }

  // Surface important lines in the in-app log view (without overwhelming it).
  let lower = line.to_lowercase();
  if lower.contains(" error ") || lower.contains("error") {
    inner.logs.push(logbus::LogLevel::Error, "codex(app-server)", line.clone());
  } else if lower.contains("warn") || lower.contains("warning") {
    inner.logs.push(logbus::LogLevel::Warn, "codex(app-server)", line.clone());
  }

  let needle = "state db missing rollout path for thread ";
  if let Some(idx) = line.find(needle) {
    let after = &line[idx + needle.len()..];
    let thread_id = after.split_whitespace().next().unwrap_or("").to_string();
    if thread_id.is_empty() {
      return;
    }

    // If Codex no longer has state for a persisted thread, drop it so the next message creates a fresh thread.
    {
      let mut resumed = inner.resumed_threads.lock().await;
      resumed.remove(&thread_id);
    }

    let mut removed_any = false;
    {
      let mut threads = inner.chat_threads.lock().await;
      let keys: Vec<i64> = threads
        .iter()
        .filter_map(|(k, v)| if v == &thread_id { Some(*k) } else { None })
        .collect();
      for k in keys {
        threads.remove(&k);
        removed_any = true;
      }
      if removed_any {
        let _ = persist_chat_threads(inner.chat_threads_path.as_ref(), &threads);
      }
    }

    if removed_any {
      inner.logs.push(
        logbus::LogLevel::Warn,
        "codex",
        "Codex thread state missing; reset stored thread mapping for a chat".to_string(),
      );
    }
  }
}

fn extract_no_rollout_thread_id(err: &str) -> Option<String> {
  // Typical response we stringify: {"code":-32600,"message":"no rollout found for thread id ..."}
  if let Ok(v) = serde_json::from_str::<Value>(err) {
    if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
      return extract_no_rollout_thread_id_from_message(msg);
    }
  }

  extract_no_rollout_thread_id_from_message(err)
}

fn extract_no_rollout_thread_id_from_message(msg: &str) -> Option<String> {
  let needle = "no rollout found for thread id ";
  msg
    .find(needle)
    .map(|idx| {
      msg[idx + needle.len()..]
      .split_whitespace()
      .next()
      .unwrap_or("")
      .to_string()
    })
    .filter(|s| !s.is_empty())
}

fn flush_turn_chunks(p: &mut PendingTurn, force: bool) {
  // Send in "a few sentences/paragraphs" chunks to clients (e.g. Telegram).
  // When not forced, wait until we have enough text to avoid spamming.
  loop {
    let start = p.sent_byte.min(p.full_text.len());
    if start >= p.full_text.len() {
      break;
    }

    let rem = &p.full_text[start..];
    if rem.trim().is_empty() {
      break;
    }

    if !force {
      let window_end = byte_index_at_char(rem, 900);
      let window = &rem[..window_end];
      let has_para_break = window.contains("\n\n");
      let enough = count_sentence_endings(window) >= 2 || window.chars().count() >= 260;
      if !has_para_break && !enough {
        break;
      }
    }

    let min_chars = if force { 1 } else { 80 };
    let Some((new_start, chunk)) = take_stream_chunk(&p.full_text, p.sent_byte, 900, min_chars) else {
      break;
    };

    if chunk.trim().is_empty() {
      p.sent_byte = new_start.min(p.full_text.len());
      continue;
    }

    let _ = p.updates_tx.send(chunk);
    p.sent_byte = new_start.min(p.full_text.len());
  }
}

fn byte_index_at_char(s: &str, max_chars: usize) -> usize {
  if max_chars == 0 {
    return 0;
  }
  for (count, (i, _)) in s.char_indices().enumerate() {
    if count == max_chars {
      return i;
    }
  }
  s.len()
}

fn take_stream_chunk(full: &str, start_byte: usize, max_chars: usize, min_chars: usize) -> Option<(usize, String)> {
  if start_byte >= full.len() {
    return None;
  }
  let rem = &full[start_byte..];
  if rem.trim().is_empty() {
    return None;
  }

  let end_byte = byte_index_at_char(rem, max_chars);

  let desired_sentences = if min_chars <= 1 { 1 } else { 2 };
  let cut = find_stream_cut(rem, end_byte.min(rem.len()), min_chars, desired_sentences, min_chars <= 1)?;
  let raw = &rem[..cut];
  // Keep leading indentation (useful for code blocks), but drop trailing whitespace/newlines.
  let chunk_text = raw.trim_end().to_string();
  let cut_byte = cut;

  if chunk_text.chars().count() < min_chars {
    return None;
  }

  // Advance, skipping whitespace so we don't get stuck on empty prefixes.
  let mut new_start = start_byte + cut_byte;
  while new_start < full.len() {
    let ch = full[new_start..].chars().next()?;
    if ch.is_whitespace() {
      new_start += ch.len_utf8();
      continue;
    }
    break;
  }

  Some((new_start, chunk_text))
}

fn count_sentence_endings(s: &str) -> usize {
  let mut count = 0usize;
  let mut iter = s.chars().peekable();
  while let Some(ch) = iter.next() {
    if ch == '.' || ch == '!' || ch == '?' || ch == '…' {
      let next = iter.peek().copied();
      if next.is_none() || next.map(|c| c.is_whitespace()).unwrap_or(false) {
        count += 1;
      }
    }
  }
  count
}

fn is_safe_chunk_boundary(rem: &str, cut: usize, force: bool) -> bool {
  if cut == 0 || cut > rem.len() {
    return false;
  }
  if !rem.is_char_boundary(cut) {
    return false;
  }

  // Look at the last non-whitespace char in the left side.
  let left = &rem[..cut];
  let before = left.chars().rev().find(|c| !c.is_whitespace());
  let after = rem[cut..].chars().next();

  // If we still have more text buffered, prefer boundaries that obviously separate tokens.
  // If we're forcing (turn completed), allow any boundary (we must flush).
  if force {
    return true;
  }

  let Some(b) = before else {
    return false;
  };

  // Good boundaries: paragraph/list separators or sentence punctuation.
  if b == '.' || b == '!' || b == '?' || b == '…' || b == ':' || b == ';' || b == ')' || b == ']' || b == '}' || b == ',' {
    return true;
  }

  // Cutting on whitespace is safe (we will trim_end on the chunk and skip whitespace when advancing).
  if left.ends_with(char::is_whitespace) {
    return true;
  }

  // If we are at the end of the currently buffered text, avoid splitting inside a word.
  // That happens when Codex streaming delta ends mid-token.
  if after.is_none() {
    return b.is_whitespace()
      || b == '.'
      || b == '!'
      || b == '?'
      || b == '…'
      || b == ':'
      || b == ';'
      || b == ')'
      || b == ']'
      || b == '}'
      || b == ',';
  }

  false
}

fn find_stream_cut(
  rem: &str,
  window_end: usize,
  min_chars: usize,
  desired_sentences: usize,
  force: bool,
) -> Option<usize> {
  let window = &rem[..window_end];
  let min_byte = byte_index_at_char(rem, min_chars.min(900));

  // 1) Prefer a paragraph break.
  if let Some(idx) = window.rfind("\n\n") {
    let cut = idx + 2;
    if cut >= min_byte && is_safe_chunk_boundary(rem, cut, force) {
      return Some(cut);
    }
  }

  // 2) Prefer cutting before a new list item starts (so the next chunk begins with "- ...").
  let list_markers = ["\n- ", "\n• ", "\n* ", "\n1. ", "\n2. ", "\n3. ", "\n4. "];
  let mut best_list_cut: Option<usize> = None;
  for m in list_markers {
    if let Some(idx) = window.rfind(m) {
      let cut = idx + 1; // keep '\n' at end of previous chunk
      if cut >= min_byte && is_safe_chunk_boundary(rem, cut, force) {
        best_list_cut = Some(best_list_cut.map(|b| b.max(cut)).unwrap_or(cut));
      }
    }
  }
  if let Some(cut) = best_list_cut {
    return Some(cut);
  }

  // 3) Cut after N completed sentences if possible.
  let mut sentence_ends: Vec<usize> = vec![];
  let mut chars = window.char_indices().peekable();
  while let Some((i, ch)) = chars.next() {
    if ch == '.' || ch == '!' || ch == '?' || ch == '…' {
      let next = chars.peek().map(|(_, c)| *c);
      if next.is_none() || next.map(|c| c.is_whitespace()).unwrap_or(false) {
        sentence_ends.push(i + ch.len_utf8());
      }
    }
  }
  if !sentence_ends.is_empty() {
    let mut candidate: Option<usize> = None;
    for (idx, end) in sentence_ends.iter().enumerate() {
      let have = idx + 1;
      if have >= desired_sentences && *end >= min_byte && is_safe_chunk_boundary(rem, *end, force) {
        candidate = Some(*end);
      }
    }
    if let Some(c) = candidate {
      return Some(c.min(window_end));
    }
  }

  // 4) Fallback: cut on last whitespace to avoid breaking words/paths.
  if window_end > 0 {
    let mut last_ws: Option<usize> = None;
    for (i, ch) in window.char_indices() {
      if ch.is_whitespace() && i >= min_byte {
        last_ws = Some(i);
      }
    }
    if let Some(i) = last_ws {
      if i > 0 && is_safe_chunk_boundary(rem, i, force) {
        return Some(i);
      }
    }
  }

  // 5) As a last resort, only cut at the window edge when forcing (turn completed).
  if force && window_end >= min_byte && is_safe_chunk_boundary(rem, window_end, true) {
    return Some(window_end);
  }

  None
}

fn load_chat_threads(app: &AppHandle) -> (Option<PathBuf>, HashMap<i64, String>) {
  let path = paths::codex_chat_threads_path(app).ok();
  let mut map = HashMap::<i64, String>::new();
  let Some(path2) = path.clone() else {
    return (None, map);
  };
  if !path2.exists() {
    return (Some(path2), map);
  }
  match fs::read_to_string(&path2) {
    Ok(raw) => match serde_json::from_str::<HashMap<i64, String>>(&raw) {
      Ok(m) => {
        map = m;
      }
      Err(e) => {
        log::info!("codex: failed to parse chat threads state: {e}");
      }
    },
    Err(e) => {
      log::info!("codex: failed to read chat threads state: {e}");
    }
  }
  (Some(path2), map)
}

fn persist_chat_threads(path: Option<&PathBuf>, map: &HashMap<i64, String>) -> Result<(), String> {
  let Some(path) = path else { return Ok(()); };
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).map_err(|e| format!("create codex state dir failed: {e}"))?;
  }
  let raw = serde_json::to_string_pretty(map).map_err(|e| format!("serialize codex state failed: {e}"))?;
  fs::write(path, raw).map_err(|e| format!("write codex state failed: {e}"))?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
  }
  Ok(())
}
