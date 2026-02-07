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
            <CardDescription>
              Long polling via getUpdates (later). Token/allowlist/config is hidden behind settings.
            </CardDescription>
          </div>

          <Dialog>
            <DialogTrigger asChild>
              <Button variant="ghost" size="icon" title="Telegram settings">
                <Settings2 className="h-5 w-5" />
              </Button>
            </DialogTrigger>

            <DialogContent className="max-w-xl">
              <DialogHeader>
                <DialogTitle>Telegram Settings</DialogTitle>
                <DialogDescription>
                  Add token once, configure allowlist and polling, then close. Token will not be shown after saving.
                </DialogDescription>
              </DialogHeader>

              <div className="space-y-4">
                <Card>
                  <CardHeader>
                    <CardTitle>Token</CardTitle>
                    <CardDescription>Stored in OS keychain / credential manager (later).</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="text-sm text-muted-foreground">
                      Status: <span className="text-foreground font-medium">{tokenStored ? 'stored' : 'missing'}</span>
                    </div>
                    <Input placeholder="Paste bot token (will not be shown after save)" />
                    <div className="flex gap-2">
                      <Button>Save token</Button>
                      <Button variant="outline">Delete token</Button>
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle>Allowlist</CardTitle>
                    <CardDescription>Only these chat IDs can use privileged commands.</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="text-sm text-muted-foreground">
                      Stage 0 placeholder. We will add CRUD and display `chat_id` flow with `/whoami`.
                    </div>
                    <div className="flex gap-2">
                      <Input placeholder="Add chat_id (e.g. 123456789)" />
                      <Button variant="outline">Add</Button>
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle>Polling</CardTitle>
                    <CardDescription>Timeout and offset persistence (later).</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <Input placeholder="pollTimeoutSec (e.g. 20)" />
                  </CardContent>
                </Card>
              </div>
            </DialogContent>
          </Dialog>
        </CardHeader>

        <CardContent className="space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-sm text-muted-foreground">
            <div>
              Bot status:{' '}
              <span className="text-foreground font-medium">{telegramRunning ? 'running' : 'stopped'}</span>
            </div>
            <div>
              Token:{' '}
              <span className="text-foreground font-medium">{tokenStored ? 'stored' : 'missing'}</span>
            </div>
            <div>
              Allowed chats:{' '}
              <span className="text-foreground font-medium">{allowedChatsCount}</span>
            </div>
          </div>

          <div className="flex gap-2">
            <Button variant="outline">Start</Button>
            <Button variant="outline">Stop</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
