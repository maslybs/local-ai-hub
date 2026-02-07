import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export function MemoryView() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Memory</h2>
        <p className="text-muted-foreground">
          Local database and data management UI (later).
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Database</CardTitle>
          <CardDescription>Schema, browsing, cleanup, backups (later).</CardDescription>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Placeholder. We will add storage + views for items, search, and retention rules.
        </CardContent>
      </Card>
    </div>
  );
}

