import React from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { View } from '../types';
import { useI18n } from '@/i18n/I18nContext';

type OverviewViewProps = {
  telegramRunning: boolean;
  tokenStored: boolean;
  allowedChatsCount: number;
  codexReady: boolean;
  onNavigate: (v: View) => void;
};

export function OverviewView({
  telegramRunning,
  tokenStored,
  allowedChatsCount,
  codexReady,
  onNavigate,
}: OverviewViewProps) {
  const { t } = useI18n();
  return (
    <div className="space-y-6">
      <div className="flex items-baseline justify-between gap-4">
        <h2 className="text-2xl font-bold">{t('overview.title')}</h2>
        <Badge variant="secondary">{t('common.beta')}</Badge>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>{t('overview.ai_core')}</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center gap-2">
              <Badge variant={codexReady ? 'success' : 'warning'}>
                {codexReady ? t('overview.codex_ready') : t('overview.setup_needed')}
              </Badge>
              <Badge variant="outline">{t('overview.skills_count', { count: 0 })}</Badge>
            </div>
            <Button variant="outline" onClick={() => onNavigate('ai_core')}>{t('common.open')}</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{t('overview.connectors')}</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant={telegramRunning ? 'success' : 'secondary'}>
                {t('overview.telegram', { state: telegramRunning ? t('common.on') : t('common.off') })}
              </Badge>
              <Badge variant={tokenStored ? 'success' : 'warning'}>{tokenStored ? t('overview.token_set') : t('overview.token_missing')}</Badge>
              <Badge variant="outline">{t('overview.allowed', { count: allowedChatsCount })}</Badge>
            </div>
            <Button variant="outline" onClick={() => onNavigate('connectors')}>{t('common.open')}</Button>
          </CardContent>
        </Card>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>{t('overview.skills')}</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center gap-2">
              <Badge variant="outline">{t('overview.installed', { count: 0 })}</Badge>
              <Badge variant="outline">{t('overview.enabled', { count: 0 })}</Badge>
            </div>
            <Button variant="outline" onClick={() => onNavigate('skills')}>{t('common.open')}</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>{t('overview.memory')}</CardTitle>
            <CardDescription />
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center gap-2">
              <Badge variant="secondary">{t('common.unavailable_beta')}</Badge>
            </div>
            <Button variant="outline" onClick={() => onNavigate('memory')}>{t('common.open')}</Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
