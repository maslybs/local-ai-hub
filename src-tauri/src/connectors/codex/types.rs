#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct CodexStatus {
  pub running: bool,
  pub initialized: bool,
  pub last_error: Option<String>,
  pub auth_mode: Option<String>,
  pub login_url: Option<String>,
  pub login_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct CodexDoctor {
  pub node_ok: bool,
  pub node_path: Option<String>,
  pub npm_ok: bool,
  pub npm_path: Option<String>,
  pub codex_ok: bool,
  pub codex_version: Option<String>,
  pub local_codex_ok: bool,
  pub local_codex_version: Option<String>,
  pub local_codex_entry: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexThreadSummary {
  pub id: String,
  #[serde(default)]
  pub title: Option<String>,
  // Best-effort preview, often the first user message.
  #[serde(default)]
  pub preview: Option<String>,
  #[serde(default)]
  pub updated_at_unix_ms: Option<u64>,
  #[serde(default)]
  pub created_at_unix_ms: Option<u64>,
  #[serde(default)]
  pub archived: bool,
  #[serde(default)]
  pub source_kind: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexThreadListResponse {
  #[serde(default)]
  pub threads: Vec<CodexThreadSummary>,
  #[serde(default)]
  pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexTranscriptItem {
  pub role: String,
  pub text: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexThreadReadResponse {
  pub id: String,
  #[serde(default)]
  pub title: Option<String>,
  #[serde(default)]
  pub preview: Option<String>,
  // Unix time in ms when thread was last updated (best-effort).
  #[serde(default)]
  pub updated_at_unix_ms: Option<u64>,
  #[serde(default)]
  pub items: Vec<CodexTranscriptItem>,
}
