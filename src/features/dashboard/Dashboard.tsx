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
import { backend, type AppConfig } from '@/lib/backend';

export function Dashboard() {
  const { theme, toggle: toggleTheme } = useTheme();

  const [view, setView] = React.useState<View>('overview');

  const [cfg, setCfg] = React.useState<AppConfig | null>(null);
  const [telegramRunning, setTelegramRunning] = React.useState(false);
  const [tokenStatus, setTokenStatus] = React.useState<{ stored: boolean; error: string | null }>({
    stored: false,
    error: null,
  });
  const [codexReady] = React.useState(false);

  const allowedChatsCount = cfg?.telegram.allowed_chat_ids?.length ?? 0;

  const refresh = React.useCallback(async () => {
    try {
      const [nextCfg, token, tgStatus] = await Promise.all([
        backend.getConfig(),
        backend.telegramTokenStatus(),
        backend.telegramStatus(),
      ]);
      setCfg(nextCfg);
      setTokenStatus({ stored: Boolean(token?.stored), error: token?.error ?? null });
      setTelegramRunning(Boolean(tgStatus?.running));
    } catch {
      // Non-tauri dev (browser) or backend not ready yet; keep UI usable.
    }
  }, []);

  React.useEffect(() => {
    refresh();
    const id = window.setInterval(refresh, 1500);
    return () => window.clearInterval(id);
  }, [refresh]);

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
                tokenStored={tokenStatus.stored}
                allowedChatsCount={allowedChatsCount}
                codexReady={codexReady}
                onNavigate={setView}
              />
            )}

            {view === 'ai_core' && <AiCoreView codexReady={codexReady} />}

            {view === 'connectors' && (
              <ConnectorsView
                tokenStored={tokenStatus.stored}
                telegramRunning={telegramRunning}
                allowedChatsCount={allowedChatsCount}
                config={cfg}
                onConfigChange={async (next) => {
                  await backend.saveConfig(next);
                  await refresh();
                }}
                onTelegramAction={async (action) => {
                  if (action === 'start') await backend.telegramStart();
                  if (action === 'stop') await backend.telegramStop();
                  await refresh();
                }}
                onTelegramToken={async (action, token) => {
                  if (action === 'set' && token) await backend.telegramSetToken(token);
                  if (action === 'delete') await backend.telegramDeleteToken();
                  await refresh();
                }}
                tokenError={tokenStatus.error}
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
