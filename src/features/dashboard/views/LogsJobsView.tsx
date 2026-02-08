import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { backend, type LogEntry } from '@/lib/backend';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useI18n } from '@/i18n/I18nContext';

export function LogsJobsView() {
  const { t } = useI18n();
  const [logs, setLogs] = React.useState<LogEntry[]>([]);

  const refresh = React.useCallback(async () => {
    try {
      const items = await backend.logsList(300);
      setLogs(items);
    } catch {
      // ignore (browser dev)
    }
  }, []);

  React.useEffect(() => {
    refresh();
    const id = window.setInterval(refresh, 1000);
    return () => window.clearInterval(id);
  }, [refresh]);

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">{t('logs.title')}</h2>
        <p className="text-muted-foreground">{t('logs.subtitle')}</p>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between gap-3">
            <CardTitle>{t('logs.logs')}</CardTitle>
            <div className="flex items-center gap-2">
              <Badge variant="outline">{logs.length}</Badge>
              <Button
                variant="outline"
                size="sm"
                onClick={async () => {
                  await backend.logsClear();
                  await refresh();
                }}
              >
                {t('logs.clear')}
              </Button>
            </div>
          </div>
          <CardDescription />
        </CardHeader>
        <CardContent className="space-y-2">
          {logs.length === 0 && <div className="text-sm text-muted-foreground">{t('logs.empty')}</div>}
          {logs.map((l, idx) => (
            <div key={idx} className="flex items-center gap-3 text-sm">
              <span className="w-20 text-muted-foreground uppercase">{l.level}</span>
              <span className="w-24 text-muted-foreground">{new Date(l.ts_unix_ms).toLocaleTimeString()}</span>
              <span className="w-28 text-muted-foreground">{l.source}</span>
              <span className="flex-1">{l.msg}</span>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}
