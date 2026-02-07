import React from 'react';
import { Settings2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
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
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-row items-start justify-between space-y-0">
          <div className="space-y-1">
            <CardTitle>Codex</CardTitle>
            <CardDescription />
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
                <DialogDescription>
                  Налаштуйте один раз і закрийте.
                </DialogDescription>
              </DialogHeader>

              <div className="space-y-4">
                <Card>
                  <CardHeader>
                    <CardTitle>Папка проєкту</CardTitle>
                    <CardDescription>Де Codex буде читати файли (опціонально).</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="text-sm text-muted-foreground">
                      Статус:{' '}
                      <span className="text-foreground font-medium">{codexReady ? 'готово' : 'не налаштовано'}</span>
                    </div>
                    <Input placeholder="Папка (повний шлях)" />
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle>Де знаходиться Codex</CardTitle>
                    <CardDescription>Заповнюйте тільки якщо Codex не знаходиться автоматично (опціонально).</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <Input placeholder="Шлях до Codex (опціонально)" />
                  </CardContent>
                </Card>

                <div className="flex justify-end gap-2">
                  <Button variant="outline">Закрити</Button>
                  <Button>Зберегти</Button>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </CardHeader>

        <CardContent className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <Card>
            <CardHeader>
              <CardTitle>Статус</CardTitle>
              <CardDescription />
            </CardHeader>
            <CardContent className="text-sm text-muted-foreground space-y-1">
              <div>
                Готово: <span className="text-foreground font-medium">{codexReady ? 'так' : 'ні'}</span>
              </div>
              <div>
                Папка проєкту: <span className="text-foreground font-medium">не задано</span>
              </div>
              <div className="pt-2 flex gap-2">
                <Button variant="outline" disabled title="Backend not wired yet">
                  Запустити
                </Button>
                <Button variant="outline" disabled title="Backend not wired yet">
                  Зупинити
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Діалог</CardTitle>
              <CardDescription />
            </CardHeader>
            <CardContent className="space-y-3">
              <Input placeholder="Ваше повідомлення..." />
              <div className="flex gap-2">
                <Button variant="outline" disabled title="Backend not wired yet">
                  Надіслати
                </Button>
              </div>
              <div className="rounded-md border bg-muted/30 p-3 text-sm text-muted-foreground min-h-24">
                Недоступно в бета-версії.
              </div>
            </CardContent>
          </Card>
        </CardContent>
      </Card>
    </div>
  );
}
