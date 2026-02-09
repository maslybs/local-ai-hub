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

export type CodexConfig = {
  // If true, use the user's global Codex profile so history is shared with Codex App/CLI.
  shared_history?: boolean;
  workspace_dir: string | null;
  universal_instructions?: string;
  universal_fallback_only?: boolean;
};

export type UiConfig = {
  // null = system default
  language: string | null;
};

export type AppConfig = {
  telegram: TelegramConfig;
  codex: CodexConfig;
  ui?: UiConfig;
};

export type TelegramStatus = {
  running: boolean;
  bot_username: string | null;
  last_poll_unix_ms: number | null;
  last_error: string | null;
};

export type LogLevel = 'info' | 'warn' | 'error';

export type LogEntry = {
  ts_unix_ms: number;
  level: LogLevel;
  source: string;
  msg: string;
};

export type TelegramSelfTestResult = {
  ok: boolean;
  bot_username: string | null;
  sent_test_message: boolean;
  error: string | null;
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
  auth_mode: 'apikey' | 'chatgpt' | null;
  login_url: string | null;
  login_id: string | null;
};

export type CodexDoctor = {
  node_ok: boolean;
  node_path: string | null;
  npm_ok: boolean;
  npm_path: string | null;
  codex_ok: boolean;
  codex_version: string | null;
  local_codex_ok: boolean;
  local_codex_version: string | null;
  local_codex_entry: string | null;
};

export type CodexThreadSummary = {
  id: string;
  title: string | null;
  preview: string | null;
  updatedAtUnixMs: number | null;
  createdAtUnixMs: number | null;
  archived: boolean;
  sourceKind: string | null;
};

export type CodexThreadListResponse = {
  threads: CodexThreadSummary[];
  nextCursor: string | null;
};

export type CodexThreadReadItem = {
  role: 'user' | 'assistant' | string;
  text: string;
};

export type CodexThreadReadResponse = {
  id: string;
  title: string | null;
  preview: string | null;
  updatedAtUnixMs?: number | null;
  inProgress?: boolean;
  items: CodexThreadReadItem[];
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

  async logsList(limit = 200): Promise<LogEntry[]> {
    const invoke = await getInvoke();
    return invoke<LogEntry[]>('logs_list', { limit });
  },

  async logsClear(): Promise<void> {
    const invoke = await getInvoke();
    await invoke<void>('logs_clear');
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

  async telegramSelfTest(): Promise<TelegramSelfTestResult> {
    const invoke = await getInvoke();
    return invoke<TelegramSelfTestResult>('telegram_self_test');
  },

  async codexStatus(): Promise<CodexStatus> {
    const invoke = await getInvoke();
    return invoke<CodexStatus>('codex_status');
  },

  async codexConnect(): Promise<CodexStatus> {
    const invoke = await getInvoke();
    return invoke<CodexStatus>('codex_connect');
  },

  async codexDoctor(): Promise<CodexDoctor> {
    const invoke = await getInvoke();
    return invoke<CodexDoctor>('codex_doctor');
  },

  async codexThreadList(limit = 30, cursor: string | null = null): Promise<CodexThreadListResponse> {
    const invoke = await getInvoke();
    return invoke<CodexThreadListResponse>('codex_thread_list', { limit, cursor });
  },

  async codexThreadRead(threadId: string, maxItems = 120): Promise<CodexThreadReadResponse> {
    const invoke = await getInvoke();
    return invoke<CodexThreadReadResponse>('codex_thread_read', { threadId, maxItems });
  },

  async codexInstall(): Promise<CodexDoctor> {
    const invoke = await getInvoke();
    return invoke<CodexDoctor>('codex_install');
  },

  async codexStop(): Promise<void> {
    const invoke = await getInvoke();
    await invoke<void>('codex_stop');
  },

  async codexLoginChatgpt(): Promise<CodexStatus> {
    const invoke = await getInvoke();
    return invoke<CodexStatus>('codex_login_chatgpt');
  },

  async codexLogout(): Promise<void> {
    const invoke = await getInvoke();
    await invoke<void>('codex_logout');
  },
};
