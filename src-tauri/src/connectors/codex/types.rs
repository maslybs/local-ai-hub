#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct CodexStatus {
  pub running: bool,
  pub initialized: bool,
  pub last_error: Option<String>,
}

