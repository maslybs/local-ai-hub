import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useI18n } from '@/i18n/I18nContext';

export function MemoryView() {
  const { t } = useI18n();
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">{t('memory.title')}</h2>
        <p className="text-muted-foreground">{t('common.unavailable_beta')}</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>{t('memory.database')}</CardTitle>
          <CardDescription />
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          {t('common.unavailable_beta')}
        </CardContent>
      </Card>
    </div>
  );
}
