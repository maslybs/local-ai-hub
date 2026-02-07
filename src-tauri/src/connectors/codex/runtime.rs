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
  sync::{oneshot, Mutex, RwLock},
};

use super::types::CodexStatus;
use crate::core::{logbus, paths};

#[derive(Clone)]
pub struct CodexRuntime {
  inner: Arc<Inner>,
}

struct Inner {
  status: RwLock<CodexStatus>,
  child: Mutex<Option<Child>>,
  stdin: Mutex<Option<BufWriter<ChildStdin>>>,
  next_id: AtomicU64,
  pending: Mutex<HashMap<u64, oneshot::Sender<Result<Value, String>>>>,
  pending_turns: Mutex<HashMap<String, PendingTurn>>,
  chat_threads: Mutex<HashMap<i64, String>>,
  resumed_threads: Mutex<HashSet<String>>,
  busy_chats: Mutex<HashSet<i64>>,
  chat_threads_path: Option<PathBuf>,
  logs: logbus::LogBus,
}

struct PendingTurn {
  chat_id: i64,
  agent_messages: Vec<String>,
  done: oneshot::Sender<Result<Vec<String>, String>>,
}

impl CodexRuntime {
  pub fn new(app: &AppHandle, logs: logbus::LogBus) -> Self {
    let (chat_threads_path, chat_threads) = load_chat_threads(app);
    Self {
      inner: Arc::new(Inner {
        status: RwLock::new(CodexStatus::default()),
        child: Mutex::new(None),
        stdin: Mutex::new(None),
        next_id: AtomicU64::new(1),
        pending: Mutex::new(HashMap::new()),
        pending_turns: Mutex::new(HashMap::new()),
        chat_threads: Mutex::new(chat_threads),
        resumed_threads: Mutex::new(HashSet::new()),
        busy_chats: Mutex::new(HashSet::new()),
        chat_threads_path,
        logs,
      }),
    }
  }

  pub async fn status(&self) -> CodexStatus {
    self.inner.status.read().await.clone()
  }

  pub async fn stop(&self) -> Result<(), String> {
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

  pub async fn ensure_started(&self) -> Result<(), String> {
    if self.inner.status.read().await.running {
      return Ok(());
    }

    self.inner.logs.push(logbus::LogLevel::Info, "codex", "starting app-server");
    let mut cmd = Command::new("codex");
    cmd
      .arg("app-server")
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to start codex app-server: {e}"))?;
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
    self.inner.logs.push(logbus::LogLevel::Info, "codex", "app-server running");

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
            let mut st = inner.status.write().await;
            st.last_error = Some(format!("codex stdout parse failed: {e}"));
          }
        }
      }

      let mut st = inner.status.write().await;
      st.running = false;
      st.initialized = false;
      if st.last_error.is_none() {
        st.last_error = Some("codex app-server stopped".to_string());
      }
    });

    tauri::async_runtime::spawn(async move {
      let mut lines = BufReader::new(stderr).lines();
      while let Ok(Some(line)) = lines.next_line().await {
        log::info!("codex(app-server): {}", line);
      }
    });

    Ok(())
  }

  pub async fn ensure_initialized(&self) -> Result<(), String> {
    self.ensure_started().await?;
    if self.inner.status.read().await.initialized {
      return Ok(());
    }

    self.inner.logs.push(logbus::LogLevel::Info, "codex", "initialize");
    let params = serde_json::json!({
      "clientInfo": {
        "name": "local-ai-hub",
        "version": env!("CARGO_PKG_VERSION"),
      }
    });
    let _ = self.send_request("initialize", params).await?;
    self.send_notification("initialized", None).await?;

    let mut st = self.inner.status.write().await;
    st.initialized = true;
    self.inner.logs.push(logbus::LogLevel::Info, "codex", "initialized");
    Ok(())
  }

  pub async fn ask_text(&self, chat_id: i64, text: &str) -> Result<String, String> {
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

    let res = self.ask_text_inner(chat_id, text).await;

    let mut busy = self.inner.busy_chats.lock().await;
    busy.remove(&chat_id);
    res
  }

  async fn ask_text_inner(&self, chat_id: i64, text: &str) -> Result<String, String> {
    self.ensure_initialized().await?;

    let thread_id = self.thread_for_chat(chat_id).await?;
    self
      .inner
      .logs
      .push(logbus::LogLevel::Info, "codex", format!("turn/start chat_id={chat_id}"));

    let params = serde_json::json!({
      "threadId": thread_id,
      "approvalPolicy": "never",
      "input": [
        { "type": "text", "text": text }
      ]
    });
    let turn_start = self.send_request("turn/start", params).await?;
    let turn_id = turn_start
      .get("turn")
      .and_then(|t| t.get("id"))
      .and_then(|id| id.as_str())
      .ok_or_else(|| "turn/start response missing turn.id".to_string())?
      .to_string();

    let (tx, rx) = oneshot::channel::<Result<Vec<String>, String>>();
    {
      let mut turns = self.inner.pending_turns.lock().await;
      turns.insert(
        turn_id.clone(),
        PendingTurn {
          chat_id,
          agent_messages: vec![],
          done: tx,
        },
      );
    }

    let done = match tokio::time::timeout(Duration::from_secs(180), rx).await {
      Ok(v) => v.map_err(|_| "Codex internal channel closed".to_string())?,
      Err(_) => {
        let mut turns = self.inner.pending_turns.lock().await;
        turns.remove(&turn_id);
        return Err("Codex timeout".to_string());
      }
    };

    let msgs = done?;
    let last = msgs.into_iter().filter(|s| !s.trim().is_empty()).last();
    Ok(last.unwrap_or_else(|| "Empty response".to_string()))
  }

  async fn thread_for_chat(&self, chat_id: i64) -> Result<String, String> {
    if let Some(t) = self.inner.chat_threads.lock().await.get(&chat_id).cloned() {
      self.resume_thread_if_needed(&t).await?;
      return Ok(t);
    }

    self
      .inner
      .logs
      .push(logbus::LogLevel::Info, "codex", format!("thread/start chat_id={chat_id}"));
    let params = serde_json::json!({
      "approvalPolicy": "never",
      "sandbox": "read-only"
    });
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
    let params = serde_json::json!({
      "threadId": thread_id,
      "approvalPolicy": "never",
      "sandbox": "read-only"
    });
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
    "item/completed" => {
      let turn_id = params.get("turnId").and_then(|v| v.as_str()).unwrap_or("").to_string();
      if turn_id.is_empty() {
        return;
      }
      let item = params.get("item").cloned().unwrap_or(Value::Null);
      if item.get("type").and_then(|t| t.as_str()) == Some("agentMessage") {
        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
          let mut turns = inner.pending_turns.lock().await;
          if let Some(p) = turns.get_mut(&turn_id) {
            p.agent_messages.push(text.to_string());
          }
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
      if let Some(p) = turns.remove(&turn_id) {
        // clear per-chat busy state even if we fail to send back on the channel
        {
          let mut busy = inner.busy_chats.lock().await;
          busy.remove(&p.chat_id);
        }

        if status == "failed" {
          let _ = p.done.send(Err(err_msg.unwrap_or_else(|| "Turn failed".to_string())));
        } else {
          let _ = p.done.send(Ok(p.agent_messages));
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
