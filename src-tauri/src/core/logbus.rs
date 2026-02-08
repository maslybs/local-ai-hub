use std::{
  collections::VecDeque,
  sync::{Arc, Mutex},
};

use super::time::now_unix_ms;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
  Info,
  Warn,
  Error,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
  pub ts_unix_ms: u128,
  pub level: LogLevel,
  pub source: String,
  pub msg: String,
}

#[derive(Clone)]
pub struct LogBus {
  inner: Arc<Mutex<Inner>>,
}

struct Inner {
  buf: VecDeque<LogEntry>,
  cap: usize,
}

impl LogBus {
  pub fn new(cap: usize) -> Self {
    Self {
      inner: Arc::new(Mutex::new(Inner {
        buf: VecDeque::new(),
        cap: cap.max(50),
      })),
    }
  }

  pub fn push(&self, level: LogLevel, source: impl Into<String>, msg: impl Into<String>) {
    let mut g = self.inner.lock().expect("logbus mutex");
    if g.buf.len() >= g.cap {
      g.buf.pop_front();
    }
    g.buf.push_back(LogEntry {
      ts_unix_ms: now_unix_ms(),
      level,
      source: source.into(),
      msg: msg.into(),
    });
  }

  pub fn list(&self, limit: usize) -> Vec<LogEntry> {
    let g = self.inner.lock().expect("logbus mutex");
    let lim = limit.min(g.buf.len()).max(1);
    g.buf.iter().rev().take(lim).cloned().collect()
  }

  pub fn clear(&self) {
    let mut g = self.inner.lock().expect("logbus mutex");
    g.buf.clear();
  }
}
