import React from 'react';
import { DashboardHeader } from './components/DashboardHeader';
import { Sidebar } from './components/Sidebar';
import { useTheme } from './hooks/useTheme';
import { View } from './types';
import { CodexView } from './views/CodexView';
import { LogsJobsView } from './views/LogsJobsView';
import { OverviewView } from './views/OverviewView';
import { TelegramView } from './views/TelegramView';

export function Dashboard() {
  const { theme, toggle: toggleTheme } = useTheme();

  const [view, setView] = React.useState<View>('overview');

  // Stage 0: mock state placeholders. Will be replaced by real config + backend status later.
  const [telegramRunning] = React.useState(false);
  const [tokenStored] = React.useState(false);
  const [allowedChatsCount] = React.useState(0);
  const [codexReady] = React.useState(false);

  return (
    <div className="flex h-screen bg-background">
      <Sidebar
        view={view}
        onViewChange={setView}
        telegramRunning={telegramRunning}
        codexReady={codexReady}
      />

      <div className="flex-1 flex flex-col overflow-hidden">
        <DashboardHeader
          view={view}
          theme={theme}
          onThemeToggle={toggleTheme}
          telegramRunning={telegramRunning}
          tokenStored={tokenStored}
          codexReady={codexReady}
        />

        <main className="flex-1 overflow-y-auto p-8">
          <div className="max-w-5xl mx-auto">
            {view === 'overview' && (
              <OverviewView
                telegramRunning={telegramRunning}
                tokenStored={tokenStored}
                allowedChatsCount={allowedChatsCount}
                codexReady={codexReady}
                onNavigate={setView}
              />
            )}

            {view === 'telegram' && (
              <TelegramView tokenStored={tokenStored} telegramRunning={telegramRunning} />
            )}

            {view === 'codex' && <CodexView codexReady={codexReady} />}

            {view === 'logs' && <LogsJobsView />}
          </div>
        </main>
      </div>
    </div>
  );
}

