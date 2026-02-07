import React from 'react';
import { Settings2 } from 'lucide-react';
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

type CodexViewProps = {
  codexReady: boolean;
};

export function CodexView({ codexReady }: CodexViewProps) {
  const [busy, setBusy] = React.useState(false);
  const [err, setErr] = React.useState<string | null>(null);
  const [status, setStatus] = React.useState<Awaited<ReturnType<typeof backend.codexStatus>> | null>(null);

  React.useEffect(() => {
    let alive = true;
    (async () => {
      try {
        const st = await backend.codexStatus();
        if (alive) setStatus(st);
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
  return (
    <div className="rounded-2xl border border-border/60 bg-card/70 backdrop-blur-xl shadow-sm overflow-hidden">
      <div className="flex items-center justify-between px-5 py-4 border-b border-border/50">
        <div className="flex items-center gap-2">
          <div className="text-base font-semibold">Codex</div>
          <Badge variant={codexReady ? 'success' : 'warning'}>{codexReady ? 'Ready' : 'Setup'}</Badge>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant={codexReady ? 'outline' : 'default'}
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
            title={codexReady ? 'Підключено' : 'Підключити Codex'}
          >
            {codexReady ? 'Підключено' : busy ? '...' : 'Підключити'}
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
          <div className="text-sm text-muted-foreground space-y-1">
            <div>
              Готово: <span className="text-foreground font-medium">{codexReady ? 'так' : 'ні'}</span>
            </div>
            <div>
              Акаунт: <span className="text-foreground font-medium">{authMode ?? 'нема'}</span>
            </div>
          </div>
          <div className="pt-1 flex gap-2">
            <Button variant="outline" disabled title="Недоступно в бета-версії">
              Запустити
            </Button>
            <Button variant="outline" disabled title="Недоступно в бета-версії">
              Зупинити
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
