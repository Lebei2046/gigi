import React, { useState, useEffect } from 'react';
import { useAppDispatch } from '@/store';
import { addLog } from '@/store/logsSlice';
import { agentMessagingClient } from '@/utils/agentMessaging';
import type { Agent } from '@/utils/agentMessaging';
import AgentCard from './components/AgentCard';
import EmptyState from './components/EmptyState';
import LoadingState from './components/LoadingState';
import AgentChat from './chat/AgentChat';
import { Card, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { RefreshCw } from 'lucide-react';


const Agents: React.FC = () => {
  const dispatch = useAppDispatch();
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
  const [activeTab, setActiveTab] = useState('list');

  useEffect(() => {
    // Initialize agent messaging client
    const initAgents = async () => {
      try {
        setLoading(true);
        // Query agent settings to discover available agents
        await agentMessagingClient.queryAgentSettings();
        
        // Register message handlers
        agentMessagingClient.registerMessageHandler('agent-settings-response', (message) => {
          if ('agents' in message) {
            setAgents(message.agents);
            dispatch(
              addLog({
                event: 'agent_discovery',
                data: `Discovered ${message.agents.length} agents`,
                type: 'info',
              })
            );
          }
          setLoading(false);
        });

        agentMessagingClient.registerMessageHandler('text', (message) => {
          dispatch(
            addLog({
              event: 'agent_message',
              data: `Received text message from agent ${message.sender.name}: ${message.content}`,
              type: 'info',
            })
          );
        });

        agentMessagingClient.registerMessageHandler('file', (message) => {
          dispatch(
            addLog({
              event: 'agent_file',
              data: `Received file message from agent ${message.sender.name}: ${message.filename}`,
              type: 'info',
            })
          );
        });

        // Get initial agents
        const initialAgents = agentMessagingClient.getAllAgents();
        if (initialAgents.length > 0) {
          setAgents(initialAgents);
        }
        setLoading(false);
      } catch (error) {
        console.error('Failed to initialize agents:', error);
        dispatch(
          addLog({
            event: 'agent_initialization_error',
            data: `Failed to initialize agents: ${error}`,
            type: 'error',
          })
        );
        setLoading(false);
      }
    };

    initAgents();
  }, [dispatch]);

  const handleRefreshAgents = async () => {
    try {
      setLoading(true);
      await agentMessagingClient.queryAgentSettings();
    } catch (error) {
      console.error('Failed to refresh agents:', error);
      dispatch(
        addLog({
          event: 'agent_refresh_error',
          data: `Failed to refresh agents: ${error}`,
          type: 'error',
        })
      );
    } finally {
      setLoading(false);
    }
  };

  const handleAgentSelect = (agent: Agent) => {
    setSelectedAgent(agent);
    setActiveTab('chat');
  };

  const handleBackToList = () => {
    setSelectedAgent(null);
    setActiveTab('list');
  };

  if (loading) {
    return <LoadingState />;
  }

  return (
    <div className="flex flex-col h-full p-4">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">Agents</h1>
        <Button
          variant="default"
          size="sm"
          onClick={handleRefreshAgents}
          className="flex items-center gap-2"
        >
          <RefreshCw className="h-4 w-4" />
          Refresh
        </Button>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1">
        <TabsList className="grid w-full grid-cols-2 mb-4">
          <TabsTrigger value="list">Agent List</TabsTrigger>
          <TabsTrigger value="chat">Chat</TabsTrigger>
        </TabsList>

        <TabsContent value="list" className="flex-1">
          {agents.length === 0 ? (
            <EmptyState />
          ) : (
            <div className="space-y-4">
              {agents.map((agent) => (
                <AgentCard
                  key={agent.id}
                  agent={agent}
                  onSelect={() => handleAgentSelect(agent)}
                />
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="chat" className="flex-1">
          {selectedAgent ? (
            <AgentChat agent={selectedAgent} onBack={handleBackToList} />
          ) : (
            <Card className="h-full flex items-center justify-center">
              <CardContent className="text-center">
                <p className="text-muted-foreground">Select an agent to start chatting</p>
              </CardContent>
            </Card>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default Agents;