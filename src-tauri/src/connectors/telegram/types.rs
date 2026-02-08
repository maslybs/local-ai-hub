use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelegramStatus {
  pub running: bool,
  pub bot_username: Option<String>,
  pub last_poll_unix_ms: Option<u128>,
  pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BotState {
  pub offset: i64,
}
