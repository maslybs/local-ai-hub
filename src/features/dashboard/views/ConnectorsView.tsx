import React from 'react';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { cn } from '@/lib/utils';
import { TelegramView } from './TelegramView';
import type { AppConfig, TelegramStatus } from '@/lib/backend';
import { useI18n } from '@/i18n/I18nContext';

type ConnectorsViewProps = {
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

export function ConnectorsView({
  tokenStored,
  telegramRunning,
  telegramStatus,
  allowedChatsCount,
  config,
  onConfigChange,
  onTelegramAction,
  onTelegramToken,
  tokenError,
}: ConnectorsViewProps) {
  const { t } = useI18n();
  const [selected, setSelected] = React.useState<'telegram' | 'coming_soon'>('telegram');
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">{t('connectors.title')}</h2>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[260px_1fr] gap-4 items-start">
        <Card>
          <CardHeader>
            <CardTitle>{t('connectors.list_title')}</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-1">
            <button
              type="button"
              onClick={() => setSelected('telegram')}
              className={cn(
                'w-full rounded-lg px-3 py-3 text-left transition-colors',
                selected === 'telegram'
                  ? 'bg-primary/10 text-foreground'
                  : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
              )}
            >
              <div className="flex items-center justify-between gap-3">
                <div className="font-medium">{t('connectors.telegram')}</div>
                <Badge variant={telegramRunning ? 'success' : 'secondary'}>
                  {telegramRunning ? t('common.on') : t('common.off')}
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
                <div className="font-medium">{t('connectors.add_connector')}</div>
                <Badge variant="secondary">{t('common.beta')}</Badge>
              </div>
            </button>
          </CardContent>
        </Card>

        <div className="min-w-0">
          {selected === 'telegram' && (
            <TelegramView
              tokenStored={tokenStored}
              telegramRunning={telegramRunning}
              telegramStatus={telegramStatus}
              allowedChatsCount={allowedChatsCount}
              config={config}
              onConfigChange={onConfigChange}
              onTelegramAction={onTelegramAction}
              onTelegramToken={onTelegramToken}
              tokenError={tokenError}
            />
          )}

          {selected === 'coming_soon' && (
            <Card>
              <CardHeader>
                <CardTitle>{t('connectors.other_connectors_title')}</CardTitle>
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
