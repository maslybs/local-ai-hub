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

type TelegramViewProps = {
  tokenStored: boolean;
  telegramRunning: boolean;
  allowedChatsCount: number;
};

export function TelegramView({ tokenStored, telegramRunning, allowedChatsCount }: TelegramViewProps) {
  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-row items-start justify-between space-y-0">
          <div className="space-y-1">
            <CardTitle>Telegram</CardTitle>
            <CardDescription />
          </div>

          <Dialog>
            <DialogTrigger asChild>
              <Button variant="ghost" size="icon" title="Telegram settings">
                <Settings2 className="h-5 w-5" />
              </Button>
            </DialogTrigger>

            <DialogContent className="max-w-xl">
              <DialogHeader>
                <DialogTitle>Налаштування Telegram</DialogTitle>
                <DialogDescription>
                  Додайте токен, налаштуйте доступ і закрийте.
                </DialogDescription>
              </DialogHeader>

              <div className="space-y-4">
                <Card>
                  <CardHeader>
                    <CardTitle>Токен</CardTitle>
                    <CardDescription>Зберігається безпечно (згодом через системне сховище).</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="text-sm text-muted-foreground">
                      Статус: <span className="text-foreground font-medium">{tokenStored ? 'збережено' : 'нема'}</span>
                    </div>
                    <Input placeholder="Вставте токен бота" />
                    <div className="flex gap-2">
                      <Button>Зберегти</Button>
                      <Button variant="outline">Видалити</Button>
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle>Доступ</CardTitle>
                    <CardDescription>Хто може користуватись ботом.</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="text-sm text-muted-foreground">
                      Поки що заглушка. Додамо зручне керування списком і команду `/whoami`.
                    </div>
                    <div className="flex gap-2">
                      <Input placeholder="Додати chat_id (наприклад 123456789)" />
                      <Button variant="outline">Додати</Button>
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle>Параметри</CardTitle>
                    <CardDescription />
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <Input placeholder="Timeout (наприклад 20)" />
                  </CardContent>
                </Card>
              </div>
            </DialogContent>
          </Dialog>
        </CardHeader>

        <CardContent className="space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-sm text-muted-foreground">
            <div>
              Статус:{' '}
              <span className="text-foreground font-medium">{telegramRunning ? 'працює' : 'зупинено'}</span>
            </div>
            <div>
              Токен:{' '}
              <span className="text-foreground font-medium">{tokenStored ? 'є' : 'нема'}</span>
            </div>
            <div>
              Доступ:{' '}
              <span className="text-foreground font-medium">{allowedChatsCount}</span>
            </div>
          </div>

          <div className="flex gap-2">
            <Button variant="outline" disabled title="Недоступно в бета-версії">
              Запустити
            </Button>
            <Button variant="outline" disabled title="Недоступно в бета-версії">
              Зупинити
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
