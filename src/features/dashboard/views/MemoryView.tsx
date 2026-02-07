import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export function MemoryView() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Memory</h2>
        <p className="text-muted-foreground">Недоступно в бета-версії.</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Database</CardTitle>
          <CardDescription />
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Недоступно в бета-версії.
        </CardContent>
      </Card>
    </div>
  );
}
