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
import { useI18n } from '@/i18n/I18nContext';
import { Switch } from '@/components/ui/switch';
import type { CodexThreadReadResponse, CodexThreadSummary } from '@/lib/backend';
import { listen } from '@tauri-apps/api/event';

type CodexViewProps = {
  codexReady: boolean;
};

export function CodexView({ codexReady }: CodexViewProps) {
  const { t } = useI18n();
  const [busy, setBusy] = React.useState(false);
  const [err, setErr] = React.useState<string | null>(null);
  const [status, setStatus] = React.useState<Awaited<ReturnType<typeof backend.codexStatus>> | null>(null);
  const [doctor, setDoctor] = React.useState<Awaited<ReturnType<typeof backend.codexDoctor>> | null>(null);
  const [workspaceDir, setWorkspaceDir] = React.useState<string>('');
  const [universalInstructions, setUniversalInstructions] = React.useState<string>('');
  const [fallbackOnly, setFallbackOnly] = React.useState<boolean>(true);
  const [sharedHistory, setSharedHistory] = React.useState<boolean>(false);
  const [threads, setThreads] = React.useState<CodexThreadSummary[] | null>(null);
  const [threadsBusy, setThreadsBusy] = React.useState(false);
  const [threadsCursor, setThreadsCursor] = React.useState<string | null>(null);
  const [threadsErr, setThreadsErr] = React.useState<string | null>(null);
  const [selectedThread, setSelectedThread] = React.useState<CodexThreadReadResponse | null>(null);
  const [threadReadBusy, setThreadReadBusy] = React.useState(false);
  const selectedThreadUpdatedAt = selectedThread?.updatedAtUnixMs ?? null;

  const refreshThreads = React.useCallback(async () => {
    setThreadsBusy(true);
    setThreadsErr(null);
    try {
      const res = await backend.codexThreadList(40, null);
      setThreads(res.threads ?? []);
      setThreadsCursor(res.nextCursor ?? null);
    } catch (e: any) {
      setThreadsErr(e?.message ?? String(e));
      setThreads(null);
      setThreadsCursor(null);
    } finally {
      setThreadsBusy(false);
    }
  }, []);

  const loadMoreThreads = React.useCallback(async () => {
    if (!threadsCursor) return;
    setThreadsBusy(true);
    setThreadsErr(null);
    try {
      const res = await backend.codexThreadList(40, threadsCursor);
      setThreads((prev) => [...(prev ?? []), ...(res.threads ?? [])]);
      setThreadsCursor(res.nextCursor ?? null);
    } catch (e: any) {
      setThreadsErr(e?.message ?? String(e));
    } finally {
      setThreadsBusy(false);
    }
  }, [threadsCursor]);

  const openThread = React.useCallback(async (id: string) => {
    setThreadReadBusy(true);
    setThreadsErr(null);
    try {
      const r = await backend.codexThreadRead(id, 140);
      setSelectedThread(r);
    } catch (e: any) {
      setThreadsErr(e?.message ?? String(e));
    } finally {
      setThreadReadBusy(false);
    }
  }, []);

  // Real-time: when this app-server completes a turn, refresh the open thread (if any).
  React.useEffect(() => {
    let unlisten: null | (() => void) = null;
    (async () => {
      try {
        const u = await listen<{ threadId?: string }>('codex://thread_changed', async (event) => {
          const threadId = event.payload?.threadId;
          if (!threadId) return;
          if (selectedThread?.id !== threadId) return;
          try {
            const r = await backend.codexThreadRead(threadId, 140);
            setSelectedThread(r);
          } catch {
            // ignore
          }
        });
        unlisten = () => u();
      } catch {
        // ignore (non-Tauri / no event system)
      }
    })();
    return () => {
      if (unlisten) unlisten();
    };
  }, [selectedThread?.id]);

  // Realtime across processes is not supported by app-server notifications (they only reflect this process).
  // To reflect updates made in another Codex client, we poll `thread/read` while the thread dialog is open.
  React.useEffect(() => {
    if (!selectedThread?.id) return;
    let stopped = false;
    let inFlight = false;
    const threadId = selectedThread.id;

    const tick = async () => {
      if (stopped) return;
      if (inFlight) return;
      inFlight = true;
      try {
        const r = await backend.codexThreadRead(threadId, 140);
        if (stopped) return;
        // Only update state if we observe a change; helps avoid flicker.
        if ((r.updatedAtUnixMs ?? null) !== selectedThreadUpdatedAt || (r.items?.length ?? 0) !== (selectedThread.items?.length ?? 0)) {
          setSelectedThread(r);
        }
      } catch {
        // ignore
      } finally {
        inFlight = false;
      }
    };

    const t = window.setInterval(tick, 1500);
    // Also do one immediate refresh to pick up latest.
    void tick();
    return () => {
      stopped = true;
      window.clearInterval(t);
    };
  }, [selectedThread?.id, selectedThreadUpdatedAt, selectedThread?.items?.length]);

  React.useEffect(() => {
    let alive = true;
    (async () => {
      try {
        const [st, cfg, doc] = await Promise.allSettled([backend.codexStatus(), backend.getConfig(), backend.codexDoctor()]);
        if (!alive) return;
        if (st.status === 'fulfilled') setStatus(st.value);
        if (cfg.status === 'fulfilled') {
          setWorkspaceDir(cfg.value?.codex?.workspace_dir ?? '');
          setUniversalInstructions(cfg.value?.codex?.universal_instructions ?? '');
          setFallbackOnly(cfg.value?.codex?.universal_fallback_only ?? true);
          setSharedHistory(Boolean(cfg.value?.codex?.shared_history));
        }
        if (doc.status === 'fulfilled') setDoctor(doc.value);
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
  const codexInstalled = Boolean(doctor?.local_codex_ok);

  // Auto-load history once per session when Codex becomes ready.
  React.useEffect(() => {
    if (!ready) return;
    if (threads !== null) return;
    void refreshThreads();
  }, [ready, threads, refreshThreads]);

  return (
    <div className="rounded-2xl border border-border/60 bg-card/70 backdrop-blur-xl shadow-sm overflow-hidden">
      <div className="flex items-center justify-between px-5 py-4 border-b border-border/50">
        <div className="flex items-center gap-2">
          <div className="text-base font-semibold">Codex</div>
          <Badge variant={ready ? 'success' : 'warning'}>{ready ? t('codex.ready') : t('codex.setup')}</Badge>
          <Badge variant={authMode ? 'outline' : 'secondary'}>{authMode ?? t('codex.sign_in')}</Badge>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant={ready ? 'outline' : 'default'}
            disabled={busy}
            onClick={async () => {
              setBusy(true);
              setErr(null);
              try {
                if (!codexInstalled) {
                  throw new Error(t('codex.not_installed'));
                }
                // If already ready, do a real reconnect (stop + connect).
                // "Connect" alone is idempotent and won't recover a wedged process.
                const st = ready
                  ? (await backend.codexStop(), await backend.codexConnect())
                  : await backend.codexConnect();
                setStatus(st);
              } catch (e: any) {
                setErr(e?.message ?? String(e));
              } finally {
                setBusy(false);
              }
            }}
            title={ready ? t('codex.reconnect') : t('codex.connect')}
          >
            {busy ? '...' : ready ? t('codex.reconnect') : t('codex.connect')}
          </Button>

          <Dialog>
            <DialogTrigger asChild>
              <Button variant="ghost" size="icon" title={t('codex.settings')}>
                <Settings2 className="h-5 w-5" />
              </Button>
            </DialogTrigger>

            <DialogContent className="max-w-xl max-h-[80vh] overflow-y-auto">
              <DialogHeader>
                <DialogTitle>{t('codex.settings')}</DialogTitle>
                <DialogDescription />
              </DialogHeader>

              <div className="space-y-3">
                {err && <div className="text-sm text-destructive">{err}</div>}

                {!codexInstalled && (
                  <div className="rounded-xl bg-muted/20 p-4">
                    <div className="text-sm font-medium">{t('codex.not_installed')}</div>
                    <div className="mt-3 flex gap-2">
                      <Button
                        variant="default"
                        disabled={busy}
                        onClick={async () => {
                          setBusy(true);
                          setErr(null);
                          try {
                            const mod = await import('@tauri-apps/plugin-dialog');
                            const ok = await (mod.ask?.(
                              t('codex.install_confirm_body'),
                              { title: t('codex.install_confirm_title'), kind: 'warning' },
                            ) ?? Promise.resolve(window.confirm(t('codex.install_confirm_body'))));
                            if (!ok) return;

                            const doc = await backend.codexInstall();
                            setDoctor(doc);
                            try {
                              const st = await backend.codexConnect();
                              setStatus(st);
                            } catch {
                              // ignore
                            }
                          } catch (e: any) {
                            setErr(e?.message ?? String(e));
                          } finally {
                            setBusy(false);
                          }
                        }}
                      >
                        {t('codex.install')}
                      </Button>
                      <Button
                        variant="outline"
                        disabled={busy}
                        onClick={async () => {
                          setBusy(true);
                          setErr(null);
                          try {
                            const doc = await backend.codexDoctor();
                            setDoctor(doc);
                          } catch (e: any) {
                            setErr(e?.message ?? String(e));
                          } finally {
                            setBusy(false);
                          }
                        }}
                      >
                        {t('codex.check')}
                      </Button>
                    </div>
                    {doctor && (
                      <div className="mt-3 text-xs text-muted-foreground">
                        Node: {doctor.node_ok ? 'ok' : 'missing'}
                        {doctor.node_path ? ` (${doctor.node_path})` : ''}
                        {' | '}
                        npm: {doctor.npm_ok ? 'ok' : 'missing'}
                        {doctor.npm_path ? ` (${doctor.npm_path})` : ''}
                      </div>
                    )}
                  </div>
                )}

                <div className="rounded-xl bg-muted/20 p-4">
                  <div className="flex items-center justify-between gap-3">
                    <div className="text-sm font-medium">{t('codex.shared_history')}</div>
                    <Switch
                      checked={sharedHistory}
                      onCheckedChange={async (checked) => {
                        setSharedHistory(checked);
                        setBusy(true);
                        setErr(null);
                        try {
                          const cfg = await backend.getConfig();
                          cfg.codex.shared_history = checked;
                          await backend.saveConfig(cfg);
                          try {
                            const st = await backend.codexConnect();
                            setStatus(st);
                          } catch {
                            // ignore
                          }
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        } finally {
                          setBusy(false);
                        }
                      }}
                    />
                  </div>
                </div>

                <div className="rounded-xl bg-muted/20 p-4">
                  <div className="text-sm font-medium">{t('codex.workspace')}</div>
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
                          cfg.codex = cfg.codex ?? {
                            shared_history: false,
                            workspace_dir: null,
                            universal_instructions: '',
                            universal_fallback_only: true,
                          };
                          cfg.codex.shared_history = sharedHistory;
                          cfg.codex.workspace_dir = workspaceDir.trim() ? workspaceDir.trim() : null;
                          cfg.codex.universal_instructions = universalInstructions;
                          cfg.codex.universal_fallback_only = fallbackOnly;
                          await backend.saveConfig(cfg);
                          try {
                            // Apply changes immediately (switching shared history restarts the server).
                            const st = await backend.codexConnect();
                            setStatus(st);
                          } catch {
                            // ignore
                          }
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        } finally {
                          setBusy(false);
                        }
                      }}
                      title="AGENTS.md береться з цієї папки"
                    >
                      {t('common.save')}
                    </Button>
                  </div>
                </div>

                <div className="rounded-xl bg-muted/20 p-4 space-y-3">
                  <div className="flex items-center justify-between gap-3">
                    <div className="text-sm font-medium">{t('codex.universal_instructions')}</div>
                    <Button
                      variant={fallbackOnly ? 'secondary' : 'outline'}
                      size="sm"
                      disabled={busy}
                      onClick={() => setFallbackOnly(v => !v)}
                      title={fallbackOnly ? 'Працює як fallback' : 'Завжди застосовується'}
                    >
                      {fallbackOnly ? t('codex.fallback') : t('codex.always')}
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
                    <div className="text-sm font-medium">{t('codex.account')}</div>
                    <Badge variant={authMode ? 'success' : 'secondary'}>{authMode ?? t('common.missing')}</Badge>
                  </div>

                  <div className="mt-3 flex gap-2">
                    <Button
                      variant="outline"
                      disabled={busy}
                      onClick={async () => {
                        setBusy(true);
                        setErr(null);
                        try {
                          if (!codexInstalled) {
                            throw new Error(t('codex.not_installed'));
                          }
                          const st = await backend.codexLoginChatgpt();
                          setStatus(st);
                          if (st.login_url) {
                            try {
                              const shell = await import('@tauri-apps/plugin-shell');
                              await shell.open(st.login_url);
                            } catch {
                              // ignore (user can still copy the URL)
                            }
                          }
                        } catch (e: any) {
                          setErr(e?.message ?? String(e));
                        } finally {
                          setBusy(false);
                        }
                      }}
                    >
                      {t('codex.login_chatgpt')}
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
                      {t('codex.logout')}
                    </Button>
                  </div>

                  {loginUrl && (
                    <div className="mt-3 space-y-2">
                      <div className="text-xs text-muted-foreground">URL</div>
                      <div className="flex gap-2">
                        <Input readOnly value={loginUrl} />
                        <Button
                          variant="outline"
                          disabled={busy}
                          onClick={async () => {
                            setErr(null);
                            try {
                              const shell = await import('@tauri-apps/plugin-shell');
                              await shell.open(loginUrl);
                            } catch (e: any) {
                              setErr(e?.message ?? String(e));
                            }
                          }}
                        >
                          {t('codex.copy_link')}
                        </Button>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      <div className="px-5 py-4 border-b border-border/50">
        <div className="flex items-center justify-between gap-3">
          <div className="text-sm font-medium">{t('codex.threads')}</div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={threadsBusy || busy || !ready}
              onClick={refreshThreads}
              title={t('common.refresh')}
            >
              {threadsBusy ? '...' : t('common.refresh')}
            </Button>
            <Button
              variant="outline"
              size="sm"
              disabled={threadsBusy || busy || !threadsCursor || !ready}
              onClick={loadMoreThreads}
              title={t('common.more')}
            >
              {t('common.more')}
            </Button>
          </div>
        </div>

        {threadsErr && <div className="mt-3 text-sm text-destructive">{threadsErr}</div>}

        {threads && threads.length === 0 && !threadsErr && (
          <div className="mt-3 text-sm text-muted-foreground">{t('codex.threads_empty')}</div>
        )}

        {threads && threads.length > 0 && (
          <div className="mt-3 max-h-80 overflow-y-auto pr-1 space-y-2">
            {threads.map((th, idx) => (
              <button
                key={th.id}
                className="w-full text-left flex items-start gap-3 rounded-lg border border-border/40 bg-background/30 px-3 py-2 hover:bg-background/40 transition-colors"
                onClick={() => openThread(th.id)}
                title={t('codex.open_thread')}
              >
                <div className="text-[11px] text-muted-foreground pt-0.5">{idx + 1}</div>
                <div className="min-w-0 flex-1">
                  <div className="text-sm truncate">
                    {th.title || th.preview || t('codex.thread_untitled')}
                  </div>
                  <div className="text-[11px] text-muted-foreground truncate">
                    {th.updatedAtUnixMs ? new Date(th.updatedAtUnixMs).toLocaleString() : ''}
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}

        {threadReadBusy && (
          <div className="mt-2 text-xs text-muted-foreground">{t('codex.loading')}</div>
        )}
      </div>

      {(err || lastError) && (
        <div className="px-5 py-3">
          {err && <div className="text-sm text-destructive">{err}</div>}
          {!err && lastError && <div className="text-sm text-destructive">{lastError}</div>}
        </div>
      )}

      <Dialog open={Boolean(selectedThread)} onOpenChange={(o) => { if (!o) setSelectedThread(null); }}>
        <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>{selectedThread?.title || selectedThread?.preview || t('codex.thread_untitled')}</DialogTitle>
            <DialogDescription />
          </DialogHeader>

          <div className="space-y-3">
            {(selectedThread?.items ?? []).length === 0 && (
              <div className="text-sm text-muted-foreground">{t('codex.thread_empty')}</div>
            )}
            <div className="max-h-[60vh] overflow-y-auto pr-1 space-y-2">
              {(selectedThread?.items ?? []).map((it, i) => (
                <div
                  key={i}
                  className={it.role === 'user'
                    ? 'rounded-xl border border-border/40 bg-background/30 px-3 py-2'
                    : 'rounded-xl border border-border/40 bg-muted/20 px-3 py-2'}
                >
                  <div className="text-[11px] text-muted-foreground mb-1">
                    {it.role === 'user' ? t('codex.you') : t('codex.assistant')}
                  </div>
                  <div className="text-sm whitespace-pre-wrap">{it.text}</div>
                </div>
              ))}
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
