import { describe, it, expect, vi } from 'vitest';
import { InMemoryAgentRegistry, AmpMessageRouter, AmpMessageFactory } from './message-router';
import { AgentInfo, TextMessage, FileMessage, AgentSettingsQuery, AgentSettingsResponse } from './types';

describe('InMemoryAgentRegistry', () => {
  it('should register and retrieve agents', () => {
    const registry = new InMemoryAgentRegistry();
    const agent: AgentInfo = {
      id: 'agent1',
      name: 'Test Agent',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online'
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
      status: 'online'
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
      status: 'online'
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
      status: 'online'
    };

    const agent2: AgentInfo = {
      id: 'agent2',
      name: 'Agent 2',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online'
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
      status: 'online'
    };

    const agent2: AgentInfo = {
      id: 'agent2',
      name: 'Agent 2',
      type: 'test',
      version: '1.0.0',
      settings: [],
      status: 'online'
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
      status: 'online'
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
      status: 'online'
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
});
