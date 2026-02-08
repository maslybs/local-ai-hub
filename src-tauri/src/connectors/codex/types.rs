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
