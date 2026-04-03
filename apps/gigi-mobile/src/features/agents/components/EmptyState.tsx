import React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Users } from 'lucide-react';

interface EmptyStateProps {
  onRefresh?: () => void;
}

const EmptyState: React.FC<EmptyStateProps> = ({ onRefresh }) => {
  return (
    <Card className="h-full flex items-center justify-center border-dashed">
      <CardHeader className="text-center">
        <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-muted">
          <Users className="h-6 w-6 text-muted-foreground" />
        </div>
        <CardTitle className="mt-4">No Agents Found</CardTitle>
        <CardDescription>
          No agents were discovered on the network. Try refreshing or check your network connection.
        </CardDescription>
      </CardHeader>
      <CardContent className="text-center">
        {onRefresh && (
          <Button variant="default" onClick={onRefresh}>
            Refresh Agents
          </Button>
        )}
      </CardContent>
    </Card>
  );
};

export default EmptyState;