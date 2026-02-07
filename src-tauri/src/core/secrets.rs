use keyring::{Entry, Error as KeyringError};

use tauri::AppHandle;

use super::{config_store::TokenStorageMode, paths};

const SERVICE: &str = "local-ai-hub";
const TELEGRAM_TOKEN_USER: &str = "telegram-bot-token";

fn telegram_entry() -> Result<Entry, String> {
  Entry::new(SERVICE, TELEGRAM_TOKEN_USER).map_err(|e| format!("keyring entry error: {e}"))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecretStatus {
  pub stored: bool,
  pub error: Option<String>,
  pub mode: String,
}

pub fn telegram_token_status(app: &AppHandle, mode: TokenStorageMode) -> Result<SecretStatus, String> {
  match mode {
    TokenStorageMode::Keychain => {
      let entry = telegram_entry()?;
      match entry.get_password() {
        Ok(pw) => Ok(SecretStatus {
          stored: !pw.trim().is_empty(),
          error: None,
          mode: "keychain".to_string(),
        }),
        Err(KeyringError::NoEntry) => Ok(SecretStatus {
          stored: false,
          error: None, // missing token is not an error
          mode: "keychain".to_string(),
        }),
        Err(e) => Ok(SecretStatus {
          stored: false,
          error: Some(format!("{e}")),
          mode: "keychain".to_string(),
        }),
      }
    }
    TokenStorageMode::File => {
      let path = paths::telegram_token_fallback_path(app)?;
      match std::fs::read_to_string(&path) {
        Ok(raw) => Ok(SecretStatus {
          stored: !raw.trim().is_empty(),
          error: None,
          mode: "file".to_string(),
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(SecretStatus {
          stored: false,
          error: None, // missing token is not an error
          mode: "file".to_string(),
        }),
        Err(e) => Ok(SecretStatus {
          stored: false,
          error: Some(format!("{e}")),
          mode: "file".to_string(),
        }),
      }
    }
  }
}

pub fn telegram_set_token(token: &str) -> Result<(), String> {
  let token = token.trim();
  if token.is_empty() {
    return Err("token is empty".to_string());
  }
  let entry = telegram_entry()?;
  entry
    .set_password(token)
    .map_err(|e| format!("set token failed: {e}"))?;

  // Read-back verify (as requested in SPEC).
  let read_back = entry
    .get_password()
    .map_err(|e| format!("read-back failed: {e}"))?;
  if read_back != token {
    return Err("token read-back verification failed".to_string());
  }
  Ok(())
}

pub fn telegram_set_token_for_mode(
  app: &AppHandle,
  mode: TokenStorageMode,
  token: &str,
) -> Result<(), String> {
  match mode {
    TokenStorageMode::Keychain => telegram_set_token(token),
    TokenStorageMode::File => {
      let token = token.trim();
      if token.is_empty() {
        return Err("token is empty".to_string());
      }
      let path = paths::telegram_token_fallback_path(app)?;
      if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create app data dir failed: {e}"))?;
      }
      std::fs::write(&path, token).map_err(|e| format!("write token failed: {e}"))?;
      #[cfg(unix)]
      {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
      }
      // read-back
      let rb = std::fs::read_to_string(&path).map_err(|e| format!("read-back failed: {e}"))?;
      if rb.trim() != token {
        return Err("token read-back verification failed".to_string());
      }
      Ok(())
    }
  }
}

pub fn telegram_delete_token(app: &AppHandle, mode: TokenStorageMode) -> Result<(), String> {
  match mode {
    TokenStorageMode::Keychain => {
      let entry = telegram_entry()?;
      match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(_) => Ok(()), // treat missing as success
      }
    }
    TokenStorageMode::File => {
      let path = paths::telegram_token_fallback_path(app)?;
      let _ = std::fs::remove_file(path);
      Ok(())
    }
  }
}

pub fn telegram_get_token(app: &AppHandle, mode: TokenStorageMode) -> Result<String, String> {
  match mode {
    TokenStorageMode::Keychain => {
      let entry = telegram_entry()?;
      match entry.get_password() {
        Ok(pw) => Ok(pw),
        Err(KeyringError::NoEntry) => Err("Telegram token missing".to_string()),
        Err(e) => Err(format!("get token failed: {e}")),
      }
    }
    TokenStorageMode::File => {
      let path = paths::telegram_token_fallback_path(app)?;
      match std::fs::read_to_string(path) {
        Ok(raw) => Ok(raw),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err("Telegram token missing".to_string()),
        Err(e) => Err(format!("get token failed: {e}")),
      }
    }
  }
}
