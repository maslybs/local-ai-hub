type InvokeFn = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

async function getInvoke(): Promise<InvokeFn> {
  // Running in browser (non-Tauri) should not hard-crash the UI.
  const w = window as any;
  const hasTauri = Boolean(w.__TAURI__ || w.__TAURI_INTERNALS__);
  if (!hasTauri) {
    throw new Error('Tauri backend not available');
  }
  const mod = await import('@tauri-apps/api/core');
  return mod.invoke as InvokeFn;
}

export type TelegramConfig = {
  allowed_chat_ids: number[];
  poll_timeout_sec: number;
  token_storage: 'keychain' | 'file';
};

export type AppConfig = {
  telegram: TelegramConfig;
};

export type TelegramStatus = {
  running: boolean;
  bot_username: string | null;
  last_poll_unix_ms: number | null;
  last_error: string | null;
};

export type SecretStatus = {
  stored: boolean;
  error: string | null;
  mode: 'keychain' | 'file';
};

export type CodexStatus = {
  running: boolean;
  initialized: boolean;
  last_error: string | null;
};

export const backend = {
  async ping(): Promise<string> {
    const invoke = await getInvoke();
    return invoke<string>('ping');
  },

  async getConfig(): Promise<AppConfig> {
    const invoke = await getInvoke();
    return invoke<AppConfig>('get_config');
  },

  async saveConfig(cfg: AppConfig): Promise<void> {
    const invoke = await getInvoke();
    await invoke<void>('save_config', { cfg });
  },

  async telegramTokenStatus(): Promise<SecretStatus> {
    const invoke = await getInvoke();
    return invoke<SecretStatus>('telegram_token_status');
  },

  async telegramSetToken(token: string): Promise<void> {
    const invoke = await getInvoke();
    const st = await invoke<SecretStatus>('telegram_set_token', { token });
    if (!st.stored) {
      throw new Error(st.error || (st.mode === 'keychain'
        ? 'Не вдалося зберегти в системному сховищі. Увімкни "Файл (fallback)" і спробуй ще раз.'
        : 'Не вдалося зберегти токен у файл.'));
    }
  },

  async telegramDeleteToken(): Promise<void> {
    const invoke = await getInvoke();
    await invoke<void>('telegram_delete_token');
  },

  async telegramStart(): Promise<void> {
    const invoke = await getInvoke();
    await invoke<void>('telegram_start');
  },

  async telegramStop(): Promise<void> {
    const invoke = await getInvoke();
    await invoke<void>('telegram_stop');
  },

  async telegramStatus(): Promise<TelegramStatus> {
    const invoke = await getInvoke();
    return invoke<TelegramStatus>('telegram_status');
  },

  async codexStatus(): Promise<CodexStatus> {
    const invoke = await getInvoke();
    return invoke<CodexStatus>('codex_status');
  },
};
