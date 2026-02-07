import React from 'react';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { cn } from '@/lib/utils';
import { TelegramView } from './TelegramView';

type ConnectorsViewProps = {
  tokenStored: boolean;
  telegramRunning: boolean;
  allowedChatsCount: number;
};

export function ConnectorsView({ tokenStored, telegramRunning, allowedChatsCount }: ConnectorsViewProps) {
  const [selected, setSelected] = React.useState<'telegram' | 'coming_soon'>('telegram');
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Connectors</h2>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[260px_1fr] gap-4 items-start">
        <Card>
          <CardHeader>
            <CardTitle>Конектори</CardTitle>
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
                <div className="font-medium">Telegram</div>
                <Badge variant={telegramRunning ? 'success' : 'secondary'}>
                  {telegramRunning ? 'On' : 'Off'}
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
                <div className="font-medium">Add connector</div>
                <Badge variant="secondary">Beta</Badge>
              </div>
            </button>
          </CardContent>
        </Card>

        <div className="min-w-0">
          {selected === 'telegram' && (
            <TelegramView
              tokenStored={tokenStored}
              telegramRunning={telegramRunning}
              allowedChatsCount={allowedChatsCount}
            />
          )}

          {selected === 'coming_soon' && (
            <Card>
              <CardHeader>
                <CardTitle>Інші конектори</CardTitle>
                <CardDescription />
              </CardHeader>
              <CardContent className="text-sm text-muted-foreground">
                Недоступно в бета-версії.
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
