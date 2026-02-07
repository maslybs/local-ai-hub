import React from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
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
      <div className="flex items-baseline justify-between gap-4">
        <h2 className="text-2xl font-bold">Local AI Hub</h2>
        <Badge variant="secondary">Beta</Badge>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>AI Core</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center gap-2">
              <Badge variant={codexReady ? 'success' : 'warning'}>
                {codexReady ? 'Codex ready' : 'Setup needed'}
              </Badge>
              <Badge variant="outline">Skills: 0</Badge>
            </div>
            <Button variant="outline" onClick={() => onNavigate('ai_core')}>Open</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Connectors</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant={telegramRunning ? 'success' : 'secondary'}>
                Telegram {telegramRunning ? 'on' : 'off'}
              </Badge>
              <Badge variant={tokenStored ? 'success' : 'warning'}>{tokenStored ? 'Token set' : 'Token missing'}</Badge>
              <Badge variant="outline">Allowed: {allowedChatsCount}</Badge>
            </div>
            <Button variant="outline" onClick={() => onNavigate('connectors')}>Open</Button>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Skills</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center gap-2">
              <Badge variant="outline">Installed: 0</Badge>
              <Badge variant="outline">Enabled: 0</Badge>
            </div>
            <Button variant="outline" onClick={() => onNavigate('skills')}>Open</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Memory</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center gap-2">
              <Badge variant="secondary">Unavailable in beta</Badge>
            </div>
            <Button variant="outline" onClick={() => onNavigate('memory')}>Open</Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
