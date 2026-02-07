import React from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';

type TelegramViewProps = {
  tokenStored: boolean;
  telegramRunning: boolean;
};

export function TelegramView({ tokenStored, telegramRunning }: TelegramViewProps) {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Telegram Connector</h2>
        <p className="text-muted-foreground">
          Stage 0: UI only. Token storage and bot start/stop will be wired in stages 3–5.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
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
            <CardTitle>Bot Control</CardTitle>
            <CardDescription>Long polling via getUpdates (later).</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="text-sm text-muted-foreground">
              Status: <span className="text-foreground font-medium">{telegramRunning ? 'running' : 'stopped'}</span>
            </div>
            <div className="flex gap-2">
              <Button variant="outline">Start</Button>
              <Button variant="outline">Stop</Button>
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Allowlist</CardTitle>
          <CardDescription>Only these chat IDs can use privileged commands.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="text-sm text-muted-foreground">
            Stage 0 placeholder. We will add CRUD and display `chat_id` flow with `/whoami`.
          </div>
          <div className="flex gap-2 max-w-xl">
            <Input placeholder="Add chat_id (e.g. 123456789)" />
            <Button variant="outline">Add</Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

