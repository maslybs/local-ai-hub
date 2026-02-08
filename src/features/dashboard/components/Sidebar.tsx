import React from 'react';
import { BrainCircuit, Cable, Database, LayoutDashboard, Sparkles } from 'lucide-react';
import { cn } from '@/lib/utils';
import { View } from '../types';
import { StatusIndicator } from './StatusIndicator';
import { useI18n } from '@/i18n/I18nContext';
import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

type SidebarProps = {
  view: View;
  onViewChange: (v: View) => void;
  telegramRunning: boolean;
  codexReady: boolean;
};

export function Sidebar({ view, onViewChange, telegramRunning, codexReady }: SidebarProps) {
  const { t, choice, setChoice } = useI18n();
  const navItems: Array<{
    id: View;
    label: string;
    icon: React.ComponentType<{ className?: string }>;
    status?: boolean;
  }> = [
    { id: 'overview', icon: LayoutDashboard, label: t('nav.overview') },
    { id: 'ai_core', icon: BrainCircuit, label: t('nav.ai_core'), status: codexReady },
    { id: 'connectors', icon: Cable, label: t('nav.connectors'), status: telegramRunning },
    { id: 'skills', icon: Sparkles, label: t('nav.skills') },
    { id: 'memory', icon: Database, label: t('nav.memory') },
  ];

  return (
    <aside className="w-60 border-r bg-card/75 backdrop-blur-xl flex flex-col">
      <div className="p-5 border-b">
        <h1 className="font-bold text-lg leading-tight">
          {t('app.name')}
          <span className="block text-xs font-normal text-muted-foreground">{t('app.tagline')}</span>
        </h1>
      </div>

      <nav className="flex-1 p-3 space-y-1">
        {navItems.map(item => (
          <button
            key={item.id}
            onClick={() => onViewChange(item.id)}
            className={cn(
              'w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-colors text-[15px]',
              view === item.id
                ? 'bg-primary text-primary-foreground'
                : 'hover:bg-muted text-muted-foreground hover:text-foreground'
            )}
          >
            <item.icon className="h-5 w-5" />
            <span className="flex-1 text-left">{item.label}</span>
            {item.status !== undefined && <StatusIndicator active={item.status} size="sm" />}
          </button>
        ))}
      </nav>

      <div className="p-3 border-t">
        <div className="flex items-center justify-between gap-2 px-2">
          <Badge variant="secondary" className="text-[10px]">{t('common.beta')}</Badge>
          <Select value={choice} onValueChange={(v) => setChoice(v as any)}>
            <SelectTrigger className="h-8 px-2 text-xs">
              <SelectValue placeholder={t('language.system')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="system">{t('language.system')}</SelectItem>
              <SelectItem value="en">{t('language.english')}</SelectItem>
              <SelectItem value="uk">{t('language.ukrainian')}</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </aside>
  );
}
