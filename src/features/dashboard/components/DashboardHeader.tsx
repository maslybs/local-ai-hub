import React from 'react';
import { Moon, Sun } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Theme } from '../hooks/useTheme';
import { View } from '../types';
import { useI18n } from '@/i18n/I18nContext';

type DashboardHeaderProps = {
  view: View;
  theme: Theme;
  onThemeToggle: () => void;
  onOpenLogs: () => void;
  logsOpen: boolean;
};

function titleKeyForView(view: View) {
  switch (view) {
    case 'overview':
      return 'nav.overview';
    case 'ai_core':
      return 'nav.ai_core';
    case 'connectors':
      return 'nav.connectors';
    case 'skills':
      return 'nav.skills';
    case 'memory':
      return 'nav.memory';
    case 'logs':
      return 'nav.logs';
  }
}

export function DashboardHeader({
  view,
  theme,
  onThemeToggle,
  onOpenLogs,
  logsOpen,
}: DashboardHeaderProps) {
  const { t } = useI18n();
  return (
    <header className="h-14 border-b flex items-center justify-between px-6">
      <div className="flex items-center gap-4">
        <div className="font-semibold">{t(titleKeyForView(view))}</div>
      </div>
      <div className="flex items-center gap-2">
        <Button
          variant={logsOpen ? 'secondary' : 'ghost'}
          onClick={onOpenLogs}
          size="sm"
        >
          {t('nav.logs')}
        </Button>
        <Button variant="ghost" size="icon" onClick={onThemeToggle} title="Toggle theme">
          {theme === 'light' ? <Moon className="h-5 w-5" /> : <Sun className="h-5 w-5" />}
        </Button>
      </div>
    </header>
  );
}
