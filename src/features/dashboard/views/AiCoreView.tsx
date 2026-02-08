import React from 'react';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { cn } from '@/lib/utils';
import { CodexView } from './CodexView';
import { useI18n } from '@/i18n/I18nContext';

type AiCoreViewProps = {
  codexReady: boolean;
};

export function AiCoreView({ codexReady }: AiCoreViewProps) {
  const { t } = useI18n();
  const [selected, setSelected] = React.useState<'codex' | 'coming_soon'>('codex');
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">{t('ai_core.title')}</h2>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[260px_1fr] gap-4 items-start">
        <Card>
          <CardHeader>
            <CardTitle>{t('ai_core.list_title')}</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-1">
            <button
              type="button"
              onClick={() => setSelected('codex')}
              className={cn(
                'w-full rounded-lg px-3 py-3 text-left transition-colors',
                selected === 'codex'
                  ? 'bg-primary/10 text-foreground'
                  : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
              )}
            >
              <div className="flex items-center justify-between gap-3">
                <div className="font-medium">{t('ai_core.codex')}</div>
                <Badge variant={codexReady ? 'success' : 'warning'}>
                  {codexReady ? t('codex.ready') : t('codex.setup')}
                </Badge>
              </div>
            </button>

            <button
              type="button"
              onClick={() => setSelected('coming_soon')}
              className={cn(
                'w-full rounded-lg px-3 py-3 text-left transition-colors',
                selected === 'coming_soon'
                  ? 'bg-muted/40 text-foreground'
                  : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
              )}
            >
              <div className="flex items-center justify-between gap-3">
                <div className="font-medium">{t('ai_core.add_ai')}</div>
                <Badge variant="secondary">{t('common.beta')}</Badge>
              </div>
            </button>
          </CardContent>
        </Card>

        <div className="min-w-0">
          {selected === 'codex' && <CodexView codexReady={codexReady} />}
          {selected === 'coming_soon' && (
            <Card>
              <CardHeader>
                <CardTitle>{t('ai_core.other_ai_title')}</CardTitle>
                <CardDescription />
              </CardHeader>
              <CardContent className="text-sm text-muted-foreground">
                {t('common.unavailable_beta')}
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
