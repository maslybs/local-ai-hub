import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { cn } from '@/lib/utils';
import { CodexView } from './CodexView';

type AiCoreViewProps = {
  codexReady: boolean;
};

export function AiCoreView({ codexReady }: AiCoreViewProps) {
  const [selected, setSelected] = React.useState<'codex' | 'coming_soon'>('codex');
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">AI Core</h2>
        <p className="text-muted-foreground">
          Центральний розділ з AI. Тут з часом зʼявляться інші моделі, але починаємо з Codex.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-[260px_1fr] gap-4 items-start">
        <Card>
          <CardHeader>
            <CardTitle>AI</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-2">
            <button
              type="button"
              onClick={() => setSelected('codex')}
              className={cn(
                'w-full rounded-lg border p-3 text-left transition-colors',
                selected === 'codex' ? 'bg-muted' : 'hover:bg-muted/60'
              )}
            >
              <div className="flex items-center gap-2">
                <div className="font-medium">Codex</div>
              </div>
              <div className="mt-1 text-xs text-muted-foreground">
                Основний AI-помічник для роботи з вашим проєктом.
              </div>
              <div className="mt-2 text-xs text-muted-foreground">
                Статус:{' '}
                <span className="text-foreground font-medium">
                  {codexReady ? 'готово' : 'не налаштовано'}
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
                <div className="font-medium">Додати AI</div>
              </div>
              <div className="mt-1 text-xs text-muted-foreground">
                Недоступно в бета-версії.
              </div>
            </button>
          </CardContent>
        </Card>

        <div className="min-w-0">
          {selected === 'codex' && <CodexView codexReady={codexReady} />}
          {selected === 'coming_soon' && (
            <Card>
              <CardHeader>
                <CardTitle>Інші AI</CardTitle>
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
