import React from 'react';
import { DashboardHeader } from './components/DashboardHeader';
import { Sidebar } from './components/Sidebar';
import { useTheme } from './hooks/useTheme';
import { View } from './types';
import { AiCoreView } from './views/AiCoreView';
import { ConnectorsView } from './views/ConnectorsView';
import { LogsJobsView } from './views/LogsJobsView';
import { MemoryView } from './views/MemoryView';
import { OverviewView } from './views/OverviewView';
import { SkillsView } from './views/SkillsView';

export function Dashboard() {
  const { theme, toggle: toggleTheme } = useTheme();

  const [view, setView] = React.useState<View>('overview');

  // Stage 0: mock state placeholders. Will be replaced by real config + backend status later.
  const [telegramRunning] = React.useState(false);
  const [tokenStored] = React.useState(false);
  const [allowedChatsCount] = React.useState(0);
  const [codexReady] = React.useState(false);

  return (
    <div className="flex h-screen app-ambient">
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
          onOpenLogs={() => setView(v => (v === 'logs' ? 'overview' : 'logs'))}
          logsOpen={view === 'logs'}
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

            {view === 'ai_core' && <AiCoreView codexReady={codexReady} />}

            {view === 'connectors' && (
              <ConnectorsView
                tokenStored={tokenStored}
                telegramRunning={telegramRunning}
                allowedChatsCount={allowedChatsCount}
              />
            )}

            {view === 'skills' && <SkillsView />}

            {view === 'memory' && <MemoryView />}

            {view === 'logs' && <LogsJobsView />}
          </div>
        </main>
      </div>
    </div>
  );
}
