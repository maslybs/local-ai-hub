import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { CodexView } from './CodexView';

type AiCoreViewProps = {
  codexReady: boolean;
};

export function AiCoreView({ codexReady }: AiCoreViewProps) {
  const [tab, setTab] = React.useState<'codex' | 'coming_soon'>('codex');
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">AI Core</h2>
        <p className="text-muted-foreground">
          The core AI runtime and shared configuration. Codex is the first core module.
        </p>
      </div>

      <div className="space-y-4">
        <Tabs value={tab} onValueChange={(v) => setTab(v as typeof tab)}>
          <TabsList>
            <TabsTrigger value="codex">Codex</TabsTrigger>
            <TabsTrigger value="coming_soon">More AIs</TabsTrigger>
          </TabsList>

          <TabsContent value="codex" className="pt-4">
            <CodexView codexReady={codexReady} />
          </TabsContent>

          <TabsContent value="coming_soon" className="pt-4">
            <Card>
              <CardHeader>
                <CardTitle>More Core AIs</CardTitle>
                <CardDescription>Place for additional AI cores (later).</CardDescription>
              </CardHeader>
              <CardContent className="text-sm text-muted-foreground">
                This section will host additional AI modules alongside Codex. The UI will become a list of installed cores
                with per-core settings.
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}
