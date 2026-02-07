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
            <CardDescription>
              Local CLI execution connector (later). Settings are hidden behind the gear.
            </CardDescription>
          </div>

          <Dialog>
            <DialogTrigger asChild>
              <Button variant="ghost" size="icon" title="Codex settings">
                <Settings2 className="h-5 w-5" />
              </Button>
            </DialogTrigger>

            <DialogContent className="max-w-xl">
              <DialogHeader>
                <DialogTitle>Codex Settings</DialogTitle>
                <DialogDescription>
                  Configure once (workspace, binary, advanced options), then close.
                </DialogDescription>
              </DialogHeader>

              <div className="space-y-4">
                <Card>
                  <CardHeader>
                    <CardTitle>Workspace</CardTitle>
                    <CardDescription>Directory (cwd) used for `codex exec/review`.</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="text-sm text-muted-foreground">
                      Status:{' '}
                      <span className="text-foreground font-medium">{codexReady ? 'ready' : 'not configured'}</span>
                    </div>
                    <Input placeholder="workspaceDir (absolute path)" />
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle>Binary</CardTitle>
                    <CardDescription>Optional path if `codex` is not in PATH.</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <Input placeholder="binaryPath (optional)" />
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle>Advanced</CardTitle>
                    <CardDescription>Model / reasoning / timeout (later).</CardDescription>
                  </CardHeader>
                  <CardContent className="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <Input placeholder="model (optional)" />
                    <Input placeholder="reasoningEffort (optional)" />
                    <Input placeholder="timeoutMs (e.g. 600000)" />
                  </CardContent>
                </Card>

                <div className="flex justify-end gap-2">
                  <Button variant="outline">Cancel</Button>
                  <Button>Save</Button>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </CardHeader>

        <CardContent className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <Card>
            <CardHeader>
              <CardTitle>Status</CardTitle>
              <CardDescription>Quick summary.</CardDescription>
            </CardHeader>
            <CardContent className="text-sm text-muted-foreground space-y-1">
              <div>
                Ready: <span className="text-foreground font-medium">{codexReady ? 'yes' : 'no'}</span>
              </div>
              <div>
                Workspace: <span className="text-foreground font-medium">not set</span>
              </div>
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
        </CardContent>
      </Card>
    </div>
  );
}
