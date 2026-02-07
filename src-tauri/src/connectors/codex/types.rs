#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct CodexStatus {
  pub running: bool,
  pub initialized: bool,
  pub last_error: Option<String>,
  pub auth_mode: Option<String>,
  pub login_url: Option<String>,
  pub login_id: Option<String>,
}
