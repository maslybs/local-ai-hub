import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export function LogsJobsView() {
  const sample = [
    { level: 'info', msg: 'UI shell started (Stage 0).' },
    { level: 'info', msg: 'Next: add Tauri backend skeleton (Stage 1).' },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Logs & Jobs</h2>
        <p className="text-muted-foreground">Недоступно в бета-версії.</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Logs</CardTitle>
          <CardDescription />
        </CardHeader>
        <CardContent className="space-y-2">
          {sample.map((l, idx) => (
            <div key={idx} className="flex items-center gap-3 text-sm">
              <span className="w-12 text-muted-foreground uppercase">{l.level}</span>
              <span className="flex-1">{l.msg}</span>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}
