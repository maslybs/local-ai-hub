import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export function SkillsView() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Skills</h2>
        <p className="text-muted-foreground">
          Shared skill registry that should be available to all connected AIs and connectors (later).
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Skill Sources</CardTitle>
          <CardDescription>Local and remote sources (later).</CardDescription>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Placeholder. This will list installed skills and allow enabling/disabling per AI.
        </CardContent>
      </Card>
    </div>
  );
}

