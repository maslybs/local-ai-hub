import React from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';

type CodexViewProps = {
  codexReady: boolean;
};

export function CodexView({ codexReady }: CodexViewProps) {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Codex Connector</h2>
        <p className="text-muted-foreground">
          Stage 0: UI only. CLI execution and output will be wired in stage 6.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Settings</CardTitle>
            <CardDescription>Workspace and binary configuration.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="text-sm text-muted-foreground">
              Status: <span className="text-foreground font-medium">{codexReady ? 'ready' : 'not configured'}</span>
            </div>
            <Input placeholder="Workspace dir (cwd)" />
            <Input placeholder="Binary path (optional)" />
            <Button>Save</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Test Run</CardTitle>
            <CardDescription>Exec/Review from UI.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Input placeholder="Prompt for codex exec..." />
            <div className="flex gap-2">
              <Button variant="outline">Exec</Button>
              <Button variant="outline">Review</Button>
            </div>
            <div className="rounded-md border bg-muted/30 p-3 text-sm text-muted-foreground min-h-24">
              Output will appear here.
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

