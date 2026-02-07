import React from 'react';
import { MessageCircle } from 'lucide-react';
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
        <p className="text-muted-foreground">
          Підключення зовнішніх інтерфейсів. Зараз є Telegram, згодом додамо інші.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[260px_1fr] gap-4 items-start">
        <Card>
          <CardHeader>
            <CardTitle>Конектори</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-2">
            <button
              type="button"
              onClick={() => setSelected('telegram')}
              className={cn(
                'w-full rounded-lg border p-3 text-left transition-colors',
                selected === 'telegram' ? 'bg-muted' : 'hover:bg-muted/60'
              )}
            >
              <div className="flex items-center gap-2">
                <MessageCircle className="h-4 w-4 text-muted-foreground" />
                <div className="font-medium">Telegram</div>
              </div>
              <div className="mt-1 text-xs text-muted-foreground">
                Керування через Telegram-бота.
              </div>
              <div className="mt-2 text-xs text-muted-foreground">
                Статус:{' '}
                <span className="text-foreground font-medium">
                  {telegramRunning ? 'працює' : 'зупинено'}
                </span>
              </div>
            </button>

            <button
              type="button"
              onClick={() => setSelected('coming_soon')}
              className={cn(
                'w-full rounded-lg border p-3 text-left transition-colors',
                selected === 'coming_soon' ? 'bg-muted' : 'hover:bg-muted/60'
              )}
            >
              <div className="flex items-center gap-2">
                <div className="font-medium">Додати конектор</div>
              </div>
              <div className="mt-1 text-xs text-muted-foreground">
                Недоступно в бета-версії.
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
                <CardDescription>Недоступно в бета-версії.</CardDescription>
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
