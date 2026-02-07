import React from 'react';
import { Moon, Sun } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Theme } from '../hooks/useTheme';
import { View } from '../types';
import { StatusIndicator } from './StatusIndicator';

type DashboardHeaderProps = {
  view: View;
  theme: Theme;
  onThemeToggle: () => void;
  telegramRunning: boolean;
  tokenStored: boolean;
  codexReady: boolean;
};

function titleForView(view: View) {
  switch (view) {
    case 'overview':
      return 'Overview';
    case 'telegram':
      return 'Telegram';
    case 'codex':
      return 'Codex';
    case 'logs':
      return 'Logs & Jobs';
  }
}

export function DashboardHeader({
  view,
  theme,
  onThemeToggle,
  telegramRunning,
  tokenStored,
  codexReady,
}: DashboardHeaderProps) {
  return (
    <header className="h-14 border-b flex items-center justify-between px-6">
      <div className="flex items-center gap-4">
        <div className="font-semibold">{titleForView(view)}</div>
        <div className="hidden md:flex items-center gap-3 text-sm text-muted-foreground">
          <span className="inline-flex items-center gap-2">
            <StatusIndicator active={telegramRunning} />
            Telegram
          </span>
          <span className="inline-flex items-center gap-2">
            <StatusIndicator active={tokenStored} />
            Token
          </span>
          <span className="inline-flex items-center gap-2">
            <StatusIndicator active={codexReady} />
            Codex
          </span>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="icon" onClick={onThemeToggle} title="Toggle theme">
          {theme === 'light' ? <Moon className="h-5 w-5" /> : <Sun className="h-5 w-5" />}
        </Button>
      </div>
    </header>
  );
}

