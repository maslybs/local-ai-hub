import React from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { View } from '../types';

type OverviewViewProps = {
  telegramRunning: boolean;
  tokenStored: boolean;
  allowedChatsCount: number;
  codexReady: boolean;
  onNavigate: (v: View) => void;
};

export function OverviewView({
  telegramRunning,
  tokenStored,
  allowedChatsCount,
  codexReady,
  onNavigate,
}: OverviewViewProps) {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Local AI Hub</h2>
        <p className="text-muted-foreground">
          Desktop hub for local connectors. Stage 0 UI shell is up; next steps will wire Tauri backend and connectors.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Telegram</CardTitle>
            <CardDescription>Bot connector status and access control.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="text-sm text-muted-foreground">
              <div>Running: <span className="text-foreground font-medium">{telegramRunning ? 'yes' : 'no'}</span></div>
              <div>Token stored: <span className="text-foreground font-medium">{tokenStored ? 'yes' : 'no'}</span></div>
              <div>Allowed chats: <span className="text-foreground font-medium">{allowedChatsCount}</span></div>
            </div>
            <Button variant="outline" onClick={() => onNavigate('telegram')}>Open Telegram</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Codex</CardTitle>
            <CardDescription>Local CLI execution connector.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="text-sm text-muted-foreground">
              <div>Ready: <span className="text-foreground font-medium">{codexReady ? 'yes' : 'no'}</span></div>
              <div>Workspace: <span className="text-foreground font-medium">not set</span></div>
            </div>
            <Button variant="outline" onClick={() => onNavigate('codex')}>Open Codex</Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

