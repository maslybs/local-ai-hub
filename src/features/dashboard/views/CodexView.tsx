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
import { Input } from '@/components/ui/input';

type CodexViewProps = {
  codexReady: boolean;
};

export function CodexView({ codexReady }: CodexViewProps) {
  return (
    <div className="rounded-2xl border border-border/60 bg-card/70 backdrop-blur-xl shadow-sm overflow-hidden">
      <div className="flex items-center justify-between px-5 py-4 border-b border-border/50">
        <div className="flex items-center gap-2">
          <div className="text-base font-semibold">Codex</div>
          <Badge variant={codexReady ? 'success' : 'warning'}>{codexReady ? 'Ready' : 'Setup'}</Badge>
        </div>

        <Dialog>
          <DialogTrigger asChild>
            <Button variant="ghost" size="icon" title="Налаштування Codex">
              <Settings2 className="h-5 w-5" />
            </Button>
          </DialogTrigger>

          <DialogContent className="max-w-xl">
            <DialogHeader>
              <DialogTitle>Налаштування Codex</DialogTitle>
              <DialogDescription>Налаштуйте один раз і закрийте.</DialogDescription>
            </DialogHeader>

            <div className="space-y-4">
              <div className="rounded-xl bg-muted/20 p-4">
                <div className="flex items-center justify-between gap-3">
                  <div className="text-sm font-medium">Папка проєкту</div>
                  <Badge variant={codexReady ? 'success' : 'warning'}>{codexReady ? 'Готово' : 'Не налаштовано'}</Badge>
                </div>
                <div className="mt-1 text-xs text-muted-foreground">Опціонально</div>
                <div className="mt-3">
                  <Input placeholder="Папка (повний шлях)" />
                </div>
              </div>

              <div className="rounded-xl bg-muted/20 p-4">
                <div className="text-sm font-medium">Шлях до Codex</div>
                <div className="mt-1 text-xs text-muted-foreground">Опціонально</div>
                <div className="mt-3">
                  <Input placeholder="Шлях до Codex" />
                </div>
              </div>

              <div className="flex justify-end gap-2">
                <Button variant="outline">Закрити</Button>
                <Button>Зберегти</Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 divide-y lg:divide-y-0 lg:divide-x divide-border/50">
        <section className="p-5 space-y-3">
          <div className="text-sm font-medium">Статус</div>
          <div className="text-sm text-muted-foreground space-y-1">
            <div>
              Готово: <span className="text-foreground font-medium">{codexReady ? 'так' : 'ні'}</span>
            </div>
            <div>
              Папка: <span className="text-foreground font-medium">не задано</span>
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
          <Input placeholder="Ваше повідомлення..." />
          <div className="flex gap-2">
            <Button variant="outline" disabled title="Недоступно в бета-версії">
              Надіслати
            </Button>
          </div>
          <div className="rounded-xl bg-muted/20 p-4 text-sm text-muted-foreground min-h-28">
            Недоступно в бета-версії.
          </div>
        </section>
      </div>
    </div>
  );
}
