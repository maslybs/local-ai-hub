import React from 'react';
import { Circle } from 'lucide-react';
import { cn } from '@/lib/utils';

export function StatusIndicator({ active, size = 'md' }: { active: boolean; size?: 'sm' | 'md' }) {
  const sizeClass = size === 'sm' ? 'h-2 w-2' : 'h-3 w-3';
  return (
    <Circle className={cn(sizeClass, 'fill-current', active ? 'text-emerald-500' : 'text-zinc-400')} />
  );
}

