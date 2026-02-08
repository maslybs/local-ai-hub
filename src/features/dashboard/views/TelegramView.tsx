import React from 'react';
import { Settings2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import type { AppConfig, TelegramStatus } from '@/lib/backend';
import { Switch } from '@/components/ui/switch';
import { backend } from '@/lib/backend';
import { useI18n } from '@/i18n/I18nContext';

type TelegramViewProps = {
  tokenStored: boolean;
  telegramRunning: boolean;
  telegramStatus: TelegramStatus | null;
  allowedChatsCount: number;
  config: AppConfig | null;
  onConfigChange: (cfg: AppConfig) => Promise<void>;
  onTelegramAction: (action: 'start' | 'stop') => Promise<void>;
  onTelegramToken: (action: 'set' | 'delete', token?: string) => Promise<void>;
  tokenError: string | null;
};

export function TelegramView({
  tokenStored,
  telegramRunning,
  telegramStatus,
  allowedChatsCount,
  config,
  onConfigChange,
  onTelegramAction,
  onTelegramToken,
  tokenError,
}: TelegramViewProps) {
  const { t } = useI18n();
  const [tokenInput, setTokenInput] = React.useState('');
  const [chatIdInput, setChatIdInput] = React.useState('');
  const [pollTimeoutInput, setPollTimeoutInput] = React.useState<string>('');
  const [err, setErr] = React.useState<string | null>(null);

  React.useEffect(() => {
    const v = config?.telegram?.poll_timeout_sec;
    if (typeof v === 'number') setPollTimeoutInput(String(v));
  }, [config?.telegram?.poll_timeout_sec]);

  const allowedIds = config?.telegram?.allowed_chat_ids ?? [];
  const storageMode = config?.telegram?.token_storage ?? 'keychain';
  const lastErr = telegramStatus?.last_error ?? null;
  const lastPoll = telegramStatus?.last_poll_unix_ms ?? null;
  const botUsername = telegramStatus?.bot_username ?? null;

  return (
    <div className="space-y-4">
      {lastErr && (
        <div className="text-sm text-destructive">
          {lastErr}
        </div>
      )}

      <div className="rounded-2xl border border-border/60 bg-card/70 backdrop-blur-xl shadow-sm overflow-hidden">
        <div className="flex items-center justify-between px-5 py-4 border-b border-border/50">
          <div className="flex items-center gap-2">
            <div className="text-base font-semibold">Telegram</div>
            <Badge variant={telegramRunning ? 'success' : 'secondary'}>{telegramRunning ? t('common.on') : t('common.off')}</Badge>
            <Badge variant={tokenStored ? 'success' : 'warning'}>{tokenStored ? t('overview.token_set') : t('overview.token_missing')}</Badge>
            {botUsername && <Badge variant="outline">@{botUsername}</Badge>}
          </div>

          <Dialog>
            <DialogTrigger asChild>
              <Button variant="ghost" size="icon" title={t('telegram.settings')}>
                <Settings2 className="h-5 w-5" />
              </Button>
            </DialogTrigger>

            <DialogContent className="max-w-xl">
              <DialogHeader>
                <DialogTitle>{t('telegram.settings')}</DialogTitle>
                <DialogDescription />
              </DialogHeader>

              <div className="space-y-3">
                {err && <div className="text-sm text-destructive">{err}</div>}
                {!err && tokenError && <div className="text-sm text-destructive">{tokenError}</div>}
                <div className="flex justify-end">
                  <Button
                    variant="outline"
                    onClick={async () => {
                      setErr(null);
                      try {
                        const r = await backend.telegramSelfTest();
                        if (!r.ok) {
                          setErr(r.error || 'Self-test failed');
                        } else if (!r.sent_test_message) {
                          setErr('Self-test OK, але не надіслав test message (allowlist порожній?).');
                        }
                      } catch (e: any) {
                        setErr(e?.message ?? String(e));
                      }
                    }}
                  >
                    {t('telegram.self_test')}
                  </Button>
                </div>

                <div className="rounded-xl bg-muted/20 p-4">
                  <div className="flex items-center justify-between gap-3">
                    <div className="text-sm font-medium">{t('telegram.token')}</div>
                    <Badge variant={tokenStored ? 'success' : 'warning'}>{tokenStored ? t('common.ok') : t('common.missing')}</Badge>
                  </div>
                  <div className="mt-3 flex items-center justify-between">
                    <div className="text-xs text-muted-foreground">{t('telegram.token_storage_file')}</div>
                    <Switch
                      checked={storageMode === 'file'}
                      onCheckedChange={async (checked) => {
                        setErr(null);
                        const next: AppConfig = config ?? {
                          telegram: { allowed_chat_ids: [], poll_timeout_sec: 20, token_storage: 'keychain' },
                          codex: { workspace_dir: null, universal_instructions: '', universal_fallback_only: true },
                          ui: { language: null },
                        };
                        next.telegram.token_storage = checked ? 'file' : 'keychain';
                        try {
                          await onConfigChange(next);
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        }
                      }}
                      title={t('telegram.less_secure')}
                    />
                  </div>
                  <div className="mt-3 flex gap-2">
                    <Input
                      placeholder={t('telegram.token_placeholder')}
                      value={tokenInput}
                      onChange={(e) => setTokenInput(e.target.value)}
                    />
                    <Button
                      onClick={async () => {
                        setErr(null);
                        try {
                          await onTelegramToken('set', tokenInput);
                          setTokenInput('');
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        }
                      }}
                    >
                      {t('common.save')}
                    </Button>
                    <Button
                      variant="outline"
                      onClick={async () => {
                        setErr(null);
                        try {
                          await onTelegramToken('delete');
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        }
                      }}
                    >
                      {t('common.delete')}
                    </Button>
                  </div>
                </div>

                <div className="rounded-xl bg-muted/20 p-4">
                  <div className="flex items-center justify-between gap-3">
                    <div className="text-sm font-medium">{t('telegram.access')}</div>
                    <Badge variant="outline">{allowedIds.length}</Badge>
                  </div>
                  <div className="mt-3 flex gap-2">
                    <Input
                      placeholder={t('telegram.chat_id')}
                      value={chatIdInput}
                      onChange={(e) => setChatIdInput(e.target.value)}
                    />
                    <Button
                      variant="outline"
                      onClick={async () => {
                        setErr(null);
                        const id = Number(chatIdInput.trim());
                        if (!Number.isFinite(id)) {
                          setErr(t('telegram.invalid_chat_id'));
                          return;
                        }
                        const next: AppConfig = config ?? {
                          telegram: { allowed_chat_ids: [], poll_timeout_sec: 20, token_storage: 'keychain' },
                          codex: { workspace_dir: null, universal_instructions: '', universal_fallback_only: true },
                          ui: { language: null },
                        };
                        const set = new Set(next.telegram.allowed_chat_ids ?? []);
                        set.add(id);
                        next.telegram.allowed_chat_ids = Array.from(set);
                        try {
                          await onConfigChange(next);
                          setChatIdInput('');
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        }
                      }}
                    >
                      {t('common.add')}
                    </Button>
                  </div>
                  {allowedIds.length > 0 && (
                    <div className="mt-3 flex flex-wrap gap-2">
                      {allowedIds.map((id) => (
                        <button
                          key={id}
                          type="button"
                          className="text-xs rounded-md border px-2 py-1 text-muted-foreground hover:text-foreground hover:bg-muted"
                          title="Remove"
                          onClick={async () => {
                            setErr(null);
                            const next: AppConfig = config ?? {
                              telegram: { allowed_chat_ids: [], poll_timeout_sec: 20, token_storage: 'keychain' },
                              codex: { workspace_dir: null, universal_instructions: '', universal_fallback_only: true },
                              ui: { language: null },
                            };
                        next.telegram.allowed_chat_ids = (next.telegram.allowed_chat_ids ?? []).filter((x) => x !== id);
                        try {
                          await onConfigChange(next);
                            } catch (e: any) {
                              setErr(e?.message ?? String(e));
                            }
                          }}
                        >
                          {id}
                        </button>
                      ))}
                    </div>
                  )}
                </div>

                <div className="rounded-xl bg-muted/20 p-4">
                  <div className="text-sm font-medium">{t('telegram.timeout')}</div>
                  <div className="mt-3 flex gap-2">
                    <Input
                      placeholder="20"
                      value={pollTimeoutInput}
                      onChange={(e) => setPollTimeoutInput(e.target.value)}
                    />
                    <Button
                      variant="outline"
                      onClick={async () => {
                        setErr(null);
                        const v = Number(pollTimeoutInput.trim());
                        if (!Number.isFinite(v) || v <= 0) {
                          setErr(t('telegram.invalid_timeout'));
                          return;
                        }
                        const next: AppConfig = config ?? {
                          telegram: { allowed_chat_ids: [], poll_timeout_sec: 20, token_storage: 'keychain' },
                          codex: { workspace_dir: null, universal_instructions: '', universal_fallback_only: true },
                          ui: { language: null },
                        };
                        next.telegram.poll_timeout_sec = Math.floor(v);
                        try {
                          await onConfigChange(next);
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        }
                      }}
                    >
                      {t('common.save')}
                    </Button>
                  </div>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </div>

        <div className="px-5 py-4 flex items-center justify-between gap-3">
          <div className="text-sm text-muted-foreground flex items-center gap-3">
            <span>
              {t('telegram.allowlist', { count: allowedChatsCount })}
            </span>
            {typeof lastPoll === 'number' && (
              <span>
                {t('telegram.last_poll', { time: new Date(Number(lastPoll)).toLocaleTimeString() })}
              </span>
            )}
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={async () => {
                setErr(null);
                try {
                  await onTelegramAction('start');
                } catch (e: any) {
                  setErr(e?.message ?? String(e));
                }
              }}
            >
              {t('telegram.start')}
            </Button>
            <Button
              variant="outline"
              onClick={async () => {
                setErr(null);
                try {
                  await onTelegramAction('stop');
                } catch (e: any) {
                  setErr(e?.message ?? String(e));
                }
              }}
            >
              {t('telegram.stop')}
            </Button>
          </div>
        </div>

        {(err || tokenError) && (
          <div className="px-5 pb-4 text-sm text-destructive">
            {err ?? tokenError}
          </div>
        )}
      </div>
    </div>
  );
}
