import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export function SkillsView() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">Skills</h2>
        <p className="text-muted-foreground">Недоступно в бета-версії.</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Skill Sources</CardTitle>
          <CardDescription />
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          Недоступно в бета-версії.
        </CardContent>
      </Card>
    </div>
  );
}
