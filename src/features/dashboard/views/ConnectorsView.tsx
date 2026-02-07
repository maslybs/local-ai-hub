import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { TelegramView } from './TelegramView';

type ConnectorsViewProps = {
  tokenStored: boolean;
  telegramRunning: boolean;
  allowedChatsCount: number;
};

export function ConnectorsView({ tokenStored, telegramRunning, allowedChatsCount }: ConnectorsViewProps) {
  const [tab, setTab] = React.useState<'telegram' | 'coming_soon'>('telegram');
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Connectors</h2>
        <p className="text-muted-foreground">
          External interfaces that can trigger local jobs. Telegram is the first connector; more will be added here.
        </p>
      </div>

      <div className="space-y-4">
        <Tabs value={tab} onValueChange={(v) => setTab(v as typeof tab)}>
          <TabsList>
            <TabsTrigger value="telegram">Telegram</TabsTrigger>
            <TabsTrigger value="coming_soon">More Connectors</TabsTrigger>
          </TabsList>

          <TabsContent value="telegram" className="pt-4">
            <TelegramView
              tokenStored={tokenStored}
              telegramRunning={telegramRunning}
              allowedChatsCount={allowedChatsCount}
            />
          </TabsContent>

          <TabsContent value="coming_soon" className="pt-4">
            <Card>
              <CardHeader>
                <CardTitle>More Connectors</CardTitle>
                <CardDescription>Slack/Discord/CLI/Webhooks etc. (later).</CardDescription>
              </CardHeader>
              <CardContent className="text-sm text-muted-foreground">
                This area will become a connector list with enable/disable, per-connector settings, and status badges.
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}
