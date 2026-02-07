import React from 'react';
import { BrainCircuit, Cable, Database, LayoutDashboard, Sparkles } from 'lucide-react';
import { cn } from '@/lib/utils';
import { View } from '../types';
import { StatusIndicator } from './StatusIndicator';

type SidebarProps = {
  view: View;
  onViewChange: (v: View) => void;
  telegramRunning: boolean;
  codexReady: boolean;
};

export function Sidebar({ view, onViewChange, telegramRunning, codexReady }: SidebarProps) {
  const navItems: Array<{
    id: View;
    label: string;
    icon: React.ComponentType<{ className?: string }>;
    status?: boolean;
  }> = [
    { id: 'overview', icon: LayoutDashboard, label: 'Overview' },
    { id: 'ai_core', icon: BrainCircuit, label: 'AI Core', status: codexReady },
    { id: 'connectors', icon: Cable, label: 'Connectors', status: telegramRunning },
    { id: 'skills', icon: Sparkles, label: 'Skills' },
    { id: 'memory', icon: Database, label: 'Memory' },
  ];

  return (
    <aside className="w-60 border-r bg-card/75 backdrop-blur-xl flex flex-col">
      <div className="p-5 border-b">
        <h1 className="font-bold text-lg leading-tight">
          Local AI Hub
          <span className="block text-xs font-normal text-muted-foreground">desktop connectors</span>
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
        <div className="px-4 py-2 text-xs text-muted-foreground">
          Beta
        </div>
      </div>
    </aside>
  );
}
