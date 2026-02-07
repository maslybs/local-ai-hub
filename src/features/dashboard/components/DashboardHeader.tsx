import React from 'react';
import { Moon, Sun } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Theme } from '../hooks/useTheme';
import { View } from '../types';

type DashboardHeaderProps = {
  view: View;
  theme: Theme;
  onThemeToggle: () => void;
  onOpenLogs: () => void;
  logsOpen: boolean;
};

function titleForView(view: View) {
  switch (view) {
    case 'overview':
      return 'Overview';
    case 'ai_core':
      return 'AI Core';
    case 'connectors':
      return 'Connectors';
    case 'skills':
      return 'Skills';
    case 'memory':
      return 'Memory';
    case 'logs':
      return 'Logs & Jobs';
  }
}

export function DashboardHeader({
  view,
  theme,
  onThemeToggle,
  onOpenLogs,
  logsOpen,
}: DashboardHeaderProps) {
  return (
    <header className="h-14 border-b flex items-center justify-between px-6">
      <div className="flex items-center gap-4">
        <div className="font-semibold">{titleForView(view)}</div>
      </div>
      <div className="flex items-center gap-2">
        <Button
          variant={logsOpen ? 'secondary' : 'ghost'}
          onClick={onOpenLogs}
          size="sm"
        >
          Logs
        </Button>
        <Button variant="ghost" size="icon" onClick={onThemeToggle} title="Toggle theme">
          {theme === 'light' ? <Moon className="h-5 w-5" /> : <Sun className="h-5 w-5" />}
        </Button>
      </div>
    </header>
  );
}
