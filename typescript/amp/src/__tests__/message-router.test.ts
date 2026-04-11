import { describe, it, expect, vi } from 'vitest';
import {
  InMemoryAgentRegistry,
  AmpMessageRouter,
  AmpMessageFactory,
} from '../message-router';
import { AgentInfo, AgentSettingsResponse } from '../types';

describe('InMemoryAgentRegistry', () => {
  it('should register and retrieve agents', () => {
    const registry = new InMemoryAgentRegistry();
    const agent: AgentInfo = {
      id: 'agent1',
      name: 'Test Agent',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    registry.registerAgent(agent);
    expect(registry.getAgentById('agent1')).toEqual(agent);
    expect(registry.getAllAgents()).toEqual([agent]);
  });

  it('should update agent status', () => {
    const registry = new InMemoryAgentRegistry();
    const agent: AgentInfo = {
      id: 'agent1',
      name: 'Test Agent',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    registry.registerAgent(agent);
    registry.updateAgentStatus('agent1', 'busy');
    const updatedAgent = registry.getAgentById('agent1');
    expect(updatedAgent?.status).toBe('busy');
  });

  it('should unregister agents', () => {
    const registry = new InMemoryAgentRegistry();
    const agent: AgentInfo = {
      id: 'agent1',
      name: 'Test Agent',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    registry.registerAgent(agent);
    expect(registry.getAllAgents()).toHaveLength(1);
    registry.unregisterAgent('agent1');
    expect(registry.getAllAgents()).toHaveLength(0);
    expect(registry.getAgentById('agent1')).toBeUndefined();
  });
});

describe('AmpMessageRouter', () => {
  it('should route text message to all agents', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const agent1: AgentInfo = {
      id: 'agent1',
      name: 'Agent 1',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    const agent2: AgentInfo = {
      id: 'agent2',
      name: 'Agent 2',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    registry.registerAgent(agent1);
    registry.registerAgent(agent2);

    const messageHandler = vi.fn();
    router.registerMessageHandler('text', messageHandler);

    const textMessage = AmpMessageFactory.createTextMessage(
      'Hello all agents',
      { type: 'all' },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    router.routeMessage(textMessage);
    expect(messageHandler).toHaveBeenCalledTimes(2);
  });

  it('should route text message to specific agents', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const agent1: AgentInfo = {
      id: 'agent1',
      name: 'Agent 1',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    const agent2: AgentInfo = {
      id: 'agent2',
      name: 'Agent 2',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    registry.registerAgent(agent1);
    registry.registerAgent(agent2);

    const messageHandler = vi.fn();
    router.registerMessageHandler('text', messageHandler);

    const textMessage = AmpMessageFactory.createTextMessage(
      'Hello Agent 1',
      { type: 'specific', agentIds: ['agent1'] },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    router.routeMessage(textMessage);
    expect(messageHandler).toHaveBeenCalledTimes(1);
    expect(messageHandler).toHaveBeenCalledWith(textMessage, 'agent1');
  });

  it('should handle agent settings query', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const agent1: AgentInfo = {
      id: 'agent1',
      name: 'Agent 1',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    registry.registerAgent(agent1);

    const responseHandler = vi.fn();
    router.registerMessageHandler('agent-settings-response', responseHandler);

    const query = AmpMessageFactory.createAgentSettingsQuery(
      { id: 'owner1', name: 'Owner', type: 'owner' },
      undefined
    );

    router.routeMessage(query);
    expect(responseHandler).toHaveBeenCalledTimes(1);
    const response = responseHandler.mock.calls[0][0] as AgentSettingsResponse;
    expect(response.type).toBe('agent-settings-response');
    expect(response.agents).toHaveLength(1);
    expect(response.agents[0].id).toBe('agent1');
  });

  it('should route file message to all agents', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const agent1: AgentInfo = {
      id: 'agent1',
      name: 'Agent 1',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    registry.registerAgent(agent1);

    const messageHandler = vi.fn();
    router.registerMessageHandler('file', messageHandler);

    const fileMessage = AmpMessageFactory.createFileMessage(
      'test.txt',
      1024,
      'hash123',
      { type: 'all' },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    router.routeMessage(fileMessage);
    expect(messageHandler).toHaveBeenCalledTimes(1);
    expect(messageHandler).toHaveBeenCalledWith(fileMessage, 'agent1');
  });
});

describe('AmpMessageFactory', () => {
  it('should create text message', () => {
    const textMessage = AmpMessageFactory.createTextMessage(
      'Hello',
      { type: 'all' },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    expect(textMessage.type).toBe('text');
    expect(textMessage.content).toBe('Hello');
    expect(textMessage.target.type).toBe('all');
    expect(textMessage.sender.id).toBe('owner1');
    expect(textMessage.sender.name).toBe('Owner');
    expect(textMessage.sender.type).toBe('owner');
    expect(textMessage.id).toMatch(/^text-/);
  });

  it('should create node text message', () => {
    const nodeTextMessage = AmpMessageFactory.createNodeTextMessage(
      'Hello node',
      'node1',
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    expect(nodeTextMessage.type).toBe('text');
    expect(nodeTextMessage.content).toBe('Hello node');
    expect(nodeTextMessage.target.type).toBe('node');
    expect(nodeTextMessage.target.nodeId).toBe('node1');
    expect(nodeTextMessage.sender.id).toBe('owner1');
    expect(nodeTextMessage.sender.name).toBe('Owner');
    expect(nodeTextMessage.sender.type).toBe('owner');
    expect(nodeTextMessage.id).toMatch(/^text-/);
  });

  it('should create node agent text message', () => {
    const nodeAgentTextMessage = AmpMessageFactory.createNodeAgentTextMessage(
      'Hello node agent',
      'node1',
      'agent1',
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    expect(nodeAgentTextMessage.type).toBe('text');
    expect(nodeAgentTextMessage.content).toBe('Hello node agent');
    expect(nodeAgentTextMessage.target.type).toBe('node-agent');
    expect(nodeAgentTextMessage.target.nodeId).toBe('node1');
    expect(nodeAgentTextMessage.target.agentIds).toEqual(['agent1']);
    expect(nodeAgentTextMessage.sender.id).toBe('owner1');
    expect(nodeAgentTextMessage.sender.name).toBe('Owner');
    expect(nodeAgentTextMessage.sender.type).toBe('owner');
    expect(nodeAgentTextMessage.id).toMatch(/^text-/);
  });

  it('should create file message', () => {
    const fileMessage = AmpMessageFactory.createFileMessage(
      'test.txt',
      1024,
      'hash123',
      { type: 'specific', agentIds: ['agent1'] },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    expect(fileMessage.type).toBe('file');
    expect(fileMessage.filename).toBe('test.txt');
    expect(fileMessage.fileSize).toBe(1024);
    expect(fileMessage.fileHash).toBe('hash123');
    expect(fileMessage.target.type).toBe('specific');
    expect(fileMessage.target.agentIds).toEqual(['agent1']);
    expect(fileMessage.sender.id).toBe('owner1');
    expect(fileMessage.sender.name).toBe('Owner');
    expect(fileMessage.sender.type).toBe('owner');
    expect(fileMessage.id).toMatch(/^file-/);
  });

  it('should create node file message', () => {
    const nodeFileMessage = AmpMessageFactory.createNodeFileMessage(
      'test.txt',
      1024,
      'hash123',
      'node1',
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    expect(nodeFileMessage.type).toBe('file');
    expect(nodeFileMessage.filename).toBe('test.txt');
    expect(nodeFileMessage.fileSize).toBe(1024);
    expect(nodeFileMessage.fileHash).toBe('hash123');
    expect(nodeFileMessage.target.type).toBe('node');
    expect(nodeFileMessage.target.nodeId).toBe('node1');
    expect(nodeFileMessage.sender.id).toBe('owner1');
    expect(nodeFileMessage.sender.name).toBe('Owner');
    expect(nodeFileMessage.sender.type).toBe('owner');
    expect(nodeFileMessage.id).toMatch(/^file-/);
  });

  it('should create node agent file message', () => {
    const nodeAgentFileMessage = AmpMessageFactory.createNodeAgentFileMessage(
      'test.txt',
      1024,
      'hash123',
      'node1',
      'agent1',
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    expect(nodeAgentFileMessage.type).toBe('file');
    expect(nodeAgentFileMessage.filename).toBe('test.txt');
    expect(nodeAgentFileMessage.fileSize).toBe(1024);
    expect(nodeAgentFileMessage.fileHash).toBe('hash123');
    expect(nodeAgentFileMessage.target.type).toBe('node-agent');
    expect(nodeAgentFileMessage.target.nodeId).toBe('node1');
    expect(nodeAgentFileMessage.target.agentIds).toEqual(['agent1']);
    expect(nodeAgentFileMessage.sender.id).toBe('owner1');
    expect(nodeAgentFileMessage.sender.name).toBe('Owner');
    expect(nodeAgentFileMessage.sender.type).toBe('owner');
    expect(nodeAgentFileMessage.id).toMatch(/^file-/);
  });

  it('should create agent settings query', () => {
    const query = AmpMessageFactory.createAgentSettingsQuery(
      { id: 'owner1', name: 'Owner', type: 'owner' },
      ['agent1']
    );

    expect(query.type).toBe('agent-settings-query');
    expect(query.agentIds).toEqual(['agent1']);
    expect(query.sender.id).toBe('owner1');
    expect(query.sender.name).toBe('Owner');
    expect(query.sender.type).toBe('owner');
    expect(query.id).toMatch(/^query-/);
  });

  it('should create node agent settings query', () => {
    const nodeQuery = AmpMessageFactory.createNodeAgentSettingsQuery(
      'node1',
      { id: 'owner1', name: 'Owner', type: 'owner' },
      ['agent1']
    );

    expect(nodeQuery.type).toBe('agent-settings-query');
    expect(nodeQuery.nodeId).toBe('node1');
    expect(nodeQuery.agentIds).toEqual(['agent1']);
    expect(nodeQuery.sender.id).toBe('owner1');
    expect(nodeQuery.sender.name).toBe('Owner');
    expect(nodeQuery.sender.type).toBe('owner');
    expect(nodeQuery.id).toMatch(/^query-/);
  });

  it('should create agent settings response', () => {
    const agents = [
      {
        id: 'agent1',
        name: 'Agent 1',
        type: 'test',
        version: '1.0.0',
        settings: [],
        status: 'online',
      },
    ];
    const response = AmpMessageFactory.createAgentSettingsResponse(agents, {
      id: 'owner1',
      name: 'Owner',
      type: 'owner',
    });

    expect(response.type).toBe('agent-settings-response');
    expect(response.agents).toEqual(agents);
    expect(response.sender.id).toBe('owner1');
    expect(response.sender.name).toBe('Owner');
    expect(response.sender.type).toBe('owner');
    expect(response.id).toMatch(/^response-/);
  });
});

// Edge case tests
describe('InMemoryAgentRegistry - Edge Cases', () => {
  it('should return undefined for non-existent agent', () => {
    const registry = new InMemoryAgentRegistry();
    expect(registry.getAgentById('non-existent')).toBeUndefined();
  });

  it('should handle unregistering non-existent agent', () => {
    const registry = new InMemoryAgentRegistry();
    // Should not throw an error
    expect(() => registry.unregisterAgent('non-existent')).not.toThrow();
  });

  it('should handle updating status for non-existent agent', () => {
    const registry = new InMemoryAgentRegistry();
    // Should not throw an error
    expect(() =>
      registry.updateAgentStatus('non-existent', 'online')
    ).not.toThrow();
  });

  it('should return empty array when no agents are registered', () => {
    const registry = new InMemoryAgentRegistry();
    expect(registry.getAllAgents()).toEqual([]);
  });
});

describe('AmpMessageRouter - Additional Tests', () => {
  it('should route node-to-node text message', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const messageHandler = vi.fn();
    router.registerMessageHandler('text', messageHandler);

    const nodeTextMessage = AmpMessageFactory.createNodeTextMessage(
      'Hello node',
      'node1',
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    router.routeMessage(nodeTextMessage);
    expect(messageHandler).toHaveBeenCalledTimes(1);
    expect(messageHandler).toHaveBeenCalledWith(nodeTextMessage, 'node1');
  });

  it('should route node-to-agent text message', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const messageHandler = vi.fn();
    router.registerMessageHandler('text', messageHandler);

    const nodeAgentTextMessage = AmpMessageFactory.createNodeAgentTextMessage(
      'Hello node agent',
      'node1',
      'agent1',
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    router.routeMessage(nodeAgentTextMessage);
    expect(messageHandler).toHaveBeenCalledTimes(1);
    expect(messageHandler).toHaveBeenCalledWith(
      nodeAgentTextMessage,
      'node1:agent1'
    );
  });

  it('should route node-to-node file message', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const messageHandler = vi.fn();
    router.registerMessageHandler('file', messageHandler);

    const nodeFileMessage = AmpMessageFactory.createNodeFileMessage(
      'test.txt',
      1024,
      'hash123',
      'node1',
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    router.routeMessage(nodeFileMessage);
    expect(messageHandler).toHaveBeenCalledTimes(1);
    expect(messageHandler).toHaveBeenCalledWith(nodeFileMessage, 'node1');
  });

  it('should route node-to-agent file message', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const messageHandler = vi.fn();
    router.registerMessageHandler('file', messageHandler);

    const nodeAgentFileMessage = AmpMessageFactory.createNodeAgentFileMessage(
      'test.txt',
      1024,
      'hash123',
      'node1',
      'agent1',
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    router.routeMessage(nodeAgentFileMessage);
    expect(messageHandler).toHaveBeenCalledTimes(1);
    expect(messageHandler).toHaveBeenCalledWith(
      nodeAgentFileMessage,
      'node1:agent1'
    );
  });

  it('should handle agent settings response', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const responseHandler = vi.fn();
    router.registerMessageHandler('agent-settings-response', responseHandler);

    const response = AmpMessageFactory.createAgentSettingsResponse(
      [
        {
          id: 'agent1',
          name: 'Agent 1',
          type: 'test',
          version: '1.0.0',
          settings: [],
          status: 'online',
        },
      ],
      { id: 'system', name: 'System', type: 'agent' }
    );

    router.routeMessage(response);
    expect(responseHandler).toHaveBeenCalledTimes(1);
    expect(responseHandler).toHaveBeenCalledWith(response, undefined);
  });

  it('should handle node-level agent settings query', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const queryHandler = vi.fn();
    router.registerMessageHandler('agent-settings-query', queryHandler);

    const nodeQuery = AmpMessageFactory.createNodeAgentSettingsQuery(
      'node1',
      { id: 'owner1', name: 'Owner', type: 'owner' },
      ['agent1']
    );

    router.routeMessage(nodeQuery);
    expect(queryHandler).toHaveBeenCalledTimes(1);
    expect(queryHandler).toHaveBeenCalledWith(nodeQuery, 'node1');
  });

  it('should register and unregister agents through router', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const agent: AgentInfo = {
      id: 'agent1',
      name: 'Test Agent',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    router.registerAgent(agent);
    expect(registry.getAgentById('agent1')).toEqual(agent);

    router.unregisterAgent('agent1');
    expect(registry.getAgentById('agent1')).toBeUndefined();
  });

  it('should handle unknown message type', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const unknownMessage = {
      type: 'unknown',
      content: 'Unknown message',
      sender: { id: 'owner1', name: 'Owner', type: 'owner' },
      timestamp: Date.now(),
      id: 'unknown-123',
    };

    // Should not throw an error
    expect(() => router.routeMessage(unknownMessage as any)).not.toThrow();
  });

  it('should handle message with no registered handler', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const textMessage = AmpMessageFactory.createTextMessage(
      'Hello',
      { type: 'all' },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    // No handler registered, should not throw an error
    expect(() => router.routeMessage(textMessage)).not.toThrow();
  });

  it('should handle handler throwing an error', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const errorHandler = vi.fn(() => {
      throw new Error('Handler error');
    });
    router.registerMessageHandler('text', errorHandler);

    const textMessage = AmpMessageFactory.createTextMessage(
      'Hello',
      { type: 'all' },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    // Should not throw an error
    expect(() => router.routeMessage(textMessage)).not.toThrow();
  });

  it('should log error when handler throws', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    // Add an agent to ensure the handler is called
    const agent: AgentInfo = {
      id: 'agent1',
      name: 'Test Agent',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };
    registry.registerAgent(agent);

    const errorHandler = vi.fn(() => {
      throw new Error('Handler error');
    });
    router.registerMessageHandler('text', errorHandler);

    const textMessage = AmpMessageFactory.createTextMessage(
      'Hello',
      { type: 'specific', agentIds: ['agent1'] },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    // Should not throw an error
    expect(() => router.routeMessage(textMessage)).not.toThrow();
  });

  it('should route file message to specific agents', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const agent1: AgentInfo = {
      id: 'agent1',
      name: 'Agent 1',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    const agent2: AgentInfo = {
      id: 'agent2',
      name: 'Agent 2',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    registry.registerAgent(agent1);
    registry.registerAgent(agent2);

    const messageHandler = vi.fn();
    router.registerMessageHandler('file', messageHandler);

    const fileMessage = AmpMessageFactory.createFileMessage(
      'test.txt',
      1024,
      'hash123',
      { type: 'specific', agentIds: ['agent1'] },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    router.routeMessage(fileMessage);
    expect(messageHandler).toHaveBeenCalledTimes(1);
    expect(messageHandler).toHaveBeenCalledWith(fileMessage, 'agent1');
  });

  it('should handle agent settings query with specific agent IDs', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const agent1: AgentInfo = {
      id: 'agent1',
      name: 'Agent 1',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    const agent2: AgentInfo = {
      id: 'agent2',
      name: 'Agent 2',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online',
    };

    registry.registerAgent(agent1);
    registry.registerAgent(agent2);

    const responseHandler = vi.fn();
    router.registerMessageHandler('agent-settings-response', responseHandler);

    const query = AmpMessageFactory.createAgentSettingsQuery(
      { id: 'owner1', name: 'Owner', type: 'owner' },
      ['agent1']
    );

    router.routeMessage(query);
    expect(responseHandler).toHaveBeenCalledTimes(1);
    const response = responseHandler.mock.calls[0][0] as AgentSettingsResponse;
    expect(response.type).toBe('agent-settings-response');
    expect(response.agents).toHaveLength(1);
    expect(response.agents[0].id).toBe('agent1');
  });
});

describe('AmpMessageRouter - Edge Cases', () => {
  it('should handle routing to empty registry', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const messageHandler = vi.fn();
    router.registerMessageHandler('text', messageHandler);

    const textMessage = AmpMessageFactory.createTextMessage(
      'Hello',
      { type: 'all' },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    // Should not throw an error
    expect(() => router.routeMessage(textMessage)).not.toThrow();
    expect(messageHandler).not.toHaveBeenCalled();
  });

  it('should handle routing to non-existent agent', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const messageHandler = vi.fn();
    router.registerMessageHandler('text', messageHandler);

    const textMessage = AmpMessageFactory.createTextMessage(
      'Hello',
      { type: 'specific', agentIds: ['non-existent'] },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    // Should not throw an error
    expect(() => router.routeMessage(textMessage)).not.toThrow();
    expect(messageHandler).not.toHaveBeenCalled();
  });

  it('should handle routing to offline agent', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const offlineAgent: AgentInfo = {
      id: 'agent1',
      name: 'Offline Agent',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'offline',
    };

    registry.registerAgent(offlineAgent);

    const messageHandler = vi.fn();
    router.registerMessageHandler('text', messageHandler);

    const textMessage = AmpMessageFactory.createTextMessage(
      'Hello',
      { type: 'all' },
      { id: 'owner1', name: 'Owner', type: 'owner' }
    );

    // Should not throw an error
    expect(() => router.routeMessage(textMessage)).not.toThrow();
    // Message should not be routed to offline agent
    expect(messageHandler).not.toHaveBeenCalled();
  });

  it('should handle agent settings query with empty registry', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    const responseHandler = vi.fn();
    router.registerMessageHandler('agent-settings-response', responseHandler);

    const query = AmpMessageFactory.createAgentSettingsQuery(
      { id: 'owner1', name: 'Owner', type: 'owner' },
      undefined
    );

    // Should not throw an error
    expect(() => router.routeMessage(query)).not.toThrow();
    expect(responseHandler).toHaveBeenCalledTimes(1);
    const response = responseHandler.mock.calls[0][0] as AgentSettingsResponse;
    expect(response.agents).toEqual([]);
  });

  it('should handle unregistering non-existent message handler', () => {
    const registry = new InMemoryAgentRegistry();
    const router = new AmpMessageRouter(registry);

    // Should not throw an error
    expect(() => router.unregisterMessageHandler('text')).not.toThrow();
  });
});
