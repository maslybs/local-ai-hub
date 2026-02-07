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
          Desktop hub for local connectors and AI core. Stage 0 UI shell is up; next steps will wire Tauri backend and connectors.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>AI Core</CardTitle>
            <CardDescription>Core runtime shared by all connectors and skills.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="text-sm text-muted-foreground">
              <div>Codex: <span className="text-foreground font-medium">{codexReady ? 'ready' : 'not configured'}</span></div>
              <div>Shared skills: <span className="text-foreground font-medium">0</span></div>
            </div>
            <Button variant="outline" onClick={() => onNavigate('ai_core')}>Open AI Core</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Connectors</CardTitle>
            <CardDescription>External interfaces (Telegram now, more later).</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="text-sm text-muted-foreground">
              <div>Telegram running: <span className="text-foreground font-medium">{telegramRunning ? 'yes' : 'no'}</span></div>
              <div>Token stored: <span className="text-foreground font-medium">{tokenStored ? 'yes' : 'no'}</span></div>
              <div>Allowed chats: <span className="text-foreground font-medium">{allowedChatsCount}</span></div>
            </div>
            <Button variant="outline" onClick={() => onNavigate('connectors')}>Open Connectors</Button>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Skills</CardTitle>
            <CardDescription>Reusable skills available across connected AIs.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="text-sm text-muted-foreground">
              <div>Installed: <span className="text-foreground font-medium">0</span></div>
              <div>Enabled: <span className="text-foreground font-medium">0</span></div>
            </div>
            <Button variant="outline" onClick={() => onNavigate('skills')}>Open Skills</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Memory</CardTitle>
            <CardDescription>Local database and data management.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="text-sm text-muted-foreground">
              <div>Storage: <span className="text-foreground font-medium">not configured</span></div>
              <div>Items: <span className="text-foreground font-medium">0</span></div>
            </div>
            <Button variant="outline" onClick={() => onNavigate('memory')}>Open Memory</Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
