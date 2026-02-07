import React from 'react';
import { FolderOpen, Settings2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { backend } from '@/lib/backend';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';

type CodexViewProps = {
  codexReady: boolean;
};

export function CodexView({ codexReady }: CodexViewProps) {
  const [busy, setBusy] = React.useState(false);
  const [err, setErr] = React.useState<string | null>(null);
  const [status, setStatus] = React.useState<Awaited<ReturnType<typeof backend.codexStatus>> | null>(null);
  const [workspaceDir, setWorkspaceDir] = React.useState<string>('');
  const [universalInstructions, setUniversalInstructions] = React.useState<string>('');
  const [fallbackOnly, setFallbackOnly] = React.useState<boolean>(true);

  React.useEffect(() => {
    let alive = true;
    (async () => {
      try {
        const [st, cfg] = await Promise.allSettled([backend.codexStatus(), backend.getConfig()]);
        if (!alive) return;
        if (st.status === 'fulfilled') setStatus(st.value);
        if (cfg.status === 'fulfilled') {
          setWorkspaceDir(cfg.value?.codex?.workspace_dir ?? '');
          setUniversalInstructions(cfg.value?.codex?.universal_instructions ?? '');
          setFallbackOnly(cfg.value?.codex?.universal_fallback_only ?? true);
        }
      } catch {
        // ignore
      }
    })();
    return () => {
      alive = false;
    };
  }, [codexReady]);

  const authMode = status?.auth_mode ?? null;
  const loginUrl = status?.login_url ?? null;
  const lastError = status?.last_error ?? null;
  const ready = Boolean(status?.initialized);
  return (
    <div className="rounded-2xl border border-border/60 bg-card/70 backdrop-blur-xl shadow-sm overflow-hidden">
      <div className="flex items-center justify-between px-5 py-4 border-b border-border/50">
        <div className="flex items-center gap-2">
          <div className="text-base font-semibold">Codex</div>
          <Badge variant={ready ? 'success' : 'warning'}>{ready ? 'Ready' : 'Setup'}</Badge>
          <Badge variant={authMode ? 'outline' : 'secondary'}>{authMode ?? 'Sign in'}</Badge>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant={ready ? 'outline' : 'default'}
            disabled={busy}
            onClick={async () => {
              setBusy(true);
              setErr(null);
              try {
                const st = await backend.codexConnect();
                setStatus(st);
              } catch (e: any) {
                setErr(e?.message ?? String(e));
              } finally {
                setBusy(false);
              }
            }}
            title={ready ? 'Підключено' : 'Підключити Codex'}
          >
            {ready ? 'Підключено' : busy ? '...' : 'Підключити'}
          </Button>

          <Dialog>
            <DialogTrigger asChild>
              <Button variant="ghost" size="icon" title="Налаштування Codex">
                <Settings2 className="h-5 w-5" />
              </Button>
            </DialogTrigger>

            <DialogContent className="max-w-xl">
              <DialogHeader>
                <DialogTitle>Налаштування Codex</DialogTitle>
                <DialogDescription>Увійдіть один раз і закрийте.</DialogDescription>
              </DialogHeader>

              <div className="space-y-3">
                {err && <div className="text-sm text-destructive">{err}</div>}

                <div className="rounded-xl bg-muted/20 p-4">
                  <div className="text-sm font-medium">Робоча папка</div>
                  <div className="mt-3 flex gap-2">
                    <Input
                      placeholder="/path/to/project"
                      value={workspaceDir}
                      onChange={(e) => setWorkspaceDir(e.target.value)}
                    />
                    <Button
                      variant="outline"
                      disabled={busy}
                      onClick={async () => {
                        setErr(null);
                        try {
                          const mod = await import('@tauri-apps/plugin-dialog');
                          const selected = await mod.open({ directory: true, multiple: false });
                          if (typeof selected === 'string' && selected.trim()) {
                            setWorkspaceDir(selected);
                          }
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        }
                      }}
                      title="Вибрати папку"
                    >
                      <FolderOpen className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="outline"
                      disabled={busy}
                      onClick={async () => {
                        setBusy(true);
                        setErr(null);
                        try {
                          const cfg = await backend.getConfig();
                          cfg.codex = cfg.codex ?? { workspace_dir: null };
                          cfg.codex.workspace_dir = workspaceDir.trim() ? workspaceDir.trim() : null;
                          cfg.codex.universal_instructions = universalInstructions;
                          cfg.codex.universal_fallback_only = fallbackOnly;
                          await backend.saveConfig(cfg);
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        } finally {
                          setBusy(false);
                        }
                      }}
                      title="AGENTS.md береться з цієї папки"
                    >
                      Зберегти
                    </Button>
                  </div>
                </div>

                <div className="rounded-xl bg-muted/20 p-4 space-y-3">
                  <div className="flex items-center justify-between gap-3">
                    <div className="text-sm font-medium">Універсальні інструкції</div>
                    <Button
                      variant={fallbackOnly ? 'secondary' : 'outline'}
                      size="sm"
                      disabled={busy}
                      onClick={() => setFallbackOnly(v => !v)}
                      title={fallbackOnly ? 'Працює як fallback' : 'Завжди застосовується'}
                    >
                      {fallbackOnly ? 'Fallback' : 'Always'}
                    </Button>
                  </div>
                  <Textarea
                    value={universalInstructions}
                    onChange={(e) => setUniversalInstructions(e.target.value)}
                    placeholder="Коротко: яка роль асистента і як відповідати."
                    className="min-h-28"
                  />
                </div>

                <div className="rounded-xl bg-muted/20 p-4">
                  <div className="flex items-center justify-between gap-3">
                    <div className="text-sm font-medium">Акаунт</div>
                    <Badge variant={authMode ? 'success' : 'secondary'}>{authMode ?? 'нема'}</Badge>
                  </div>

                  <div className="mt-3 flex gap-2">
                    <Button
                      variant="outline"
                      disabled={busy}
                      onClick={async () => {
                        setBusy(true);
                        setErr(null);
                        try {
                          const st = await backend.codexLoginChatgpt();
                          setStatus(st);
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        } finally {
                          setBusy(false);
                        }
                      }}
                    >
                      Увійти (ChatGPT)
                    </Button>
                    <Button
                      variant="outline"
                      disabled={busy}
                      onClick={async () => {
                        setBusy(true);
                        setErr(null);
                        try {
                          await backend.codexLogout();
                          const st = await backend.codexStatus();
                          setStatus(st);
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        } finally {
                          setBusy(false);
                        }
                      }}
                    >
                      Вийти
                    </Button>
                  </div>

                  {loginUrl && (
                    <div className="mt-3 space-y-2">
                      <div className="text-xs text-muted-foreground">Відкрийте цей URL у браузері:</div>
                      <Input readOnly value={loginUrl} />
                    </div>
                  )}
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 divide-y lg:divide-y-0 lg:divide-x divide-border/50">
        <section className="p-5 space-y-3">
          <div className="text-sm font-medium">Статус</div>
          {err && <div className="text-sm text-destructive">{err}</div>}
          {!err && lastError && <div className="text-sm text-destructive">{lastError}</div>}
          <div className="text-sm text-muted-foreground space-y-1">
            <div>
              Готово: <span className="text-foreground font-medium">{ready ? 'так' : 'ні'}</span>
            </div>
            <div>
              Акаунт: <span className="text-foreground font-medium">{authMode ?? 'нема'}</span>
            </div>
          </div>
          <div className="pt-1 flex gap-2">
            <Button
              variant="outline"
              disabled={busy}
              onClick={async () => {
                setBusy(true);
                setErr(null);
                try {
                  await backend.codexStop();
                  const st = await backend.codexConnect();
                  setStatus(st);
                } catch (e: any) {
                  setErr(e?.message ?? String(e));
                } finally {
                  setBusy(false);
                }
              }}
              title="Перезапустити Codex"
            >
              Restart
            </Button>
          </div>
        </section>

        <section className="p-5 space-y-3">
          <div className="text-sm font-medium">Діалог</div>
          <div className="rounded-xl bg-muted/20 p-4 text-sm text-muted-foreground min-h-28">
            Недоступно в бета-версії.
          </div>
        </section>
      </div>
    </div>
  );
}
