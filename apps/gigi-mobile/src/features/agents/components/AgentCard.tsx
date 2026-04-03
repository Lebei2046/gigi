import React from 'react';
import type { Agent } from '@/utils/agentMessaging';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Info } from 'lucide-react';

interface AgentCardProps {
  agent: Agent;
  onSelect: () => void;
}

const AgentCard: React.FC<AgentCardProps> = ({ agent, onSelect }) => {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online':
        return 'bg-green-500';
      case 'busy':
        return 'bg-yellow-500';
      case 'offline':
        return 'bg-gray-500';
      default:
        return 'bg-gray-500';
    }
  };

  return (
    <Card className="border border-border hover:shadow-md transition-shadow">
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle className="text-lg font-semibold">{agent.name}</CardTitle>
          <CardDescription>
            {agent.type} v{agent.version}
          </CardDescription>
        </div>
        <Badge className={`${getStatusColor(agent.status)} text-white`}>
          {agent.status}
        </Badge>
      </CardHeader>
      <CardContent>
        {agent.settings && agent.settings.length > 0 && (
          <div className="mt-2 space-y-2">
            <h4 className="text-sm font-medium text-muted-foreground">Settings:</h4>
            <div className="space-y-1">
              {agent.settings.slice(0, 3).map((setting, index) => (
                <div key={index} className="text-sm">
                  <span className="font-medium">{setting.name}:</span> {setting.value}
                </div>
              ))}
              {agent.settings.length > 3 && (
                <div className="text-sm text-muted-foreground">
                  + {agent.settings.length - 3} more settings
                </div>
              )}
            </div>
          </div>
        )}
        
        {agent.openclawAgents && agent.openclawAgents.length > 0 && (
          <div className="mt-4 space-y-2">
            <h4 className="text-sm font-medium text-muted-foreground">OpenClaw Agents:</h4>
            <div className="space-y-1">
              {agent.openclawAgents.map((openclawAgent) => (
                <div key={openclawAgent.id} className="text-sm">
                  <span className="font-medium">{openclawAgent.name}:</span> {openclawAgent.model} ({openclawAgent.status})
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
      <CardFooter className="flex justify-between">
        <Button variant="default" onClick={onSelect}>
          Chat
        </Button>
        <Button variant="ghost" size="sm" className="flex items-center gap-1">
          <Info className="h-4 w-4" />
          Details
        </Button>
      </CardFooter>
    </Card>
  );
};

export default AgentCard;