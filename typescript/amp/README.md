# Gigi Agent Messaging Protocol (AMP) TypeScript Library

## Overview

The Gigi Agent Messaging Protocol (AMP) TypeScript library provides a robust framework for communication between the Claw Owner and their agents through the Gigi P2P network. It enables:

- Agents sending text and file messages to the owner
- The owner sending text and file messages to specific agents or multiple agents
- The owner querying agent settings and receiving responses
- Ensuring the owner can identify which agent sent messages

## Installation

```bash
pnpm add @gigi/amp
```

## Usage

### Basic Setup

```typescript
import {
  InMemoryAgentRegistry,
  AmpMessageRouter,
  AmpMessageFactory,
} from '@gigi/amp';

// Create agent registry
const agentRegistry = new InMemoryAgentRegistry();

// Create message router
const messageRouter = new AmpMessageRouter(agentRegistry);

// Register message handlers
messageRouter.registerMessageHandler('text', (message, agentId) => {
  console.log(
    `Received text message: ${message.content} from ${message.sender.name} (${message.sender.id})`
  );
  if (agentId) {
    console.log(`Message routed to agent: ${agentId}`);
  }
});

messageRouter.registerMessageHandler('file', (message, agentId) => {
  console.log(
    `Received file message: ${message.filename} (${message.fileSize} bytes) from ${message.sender.name}`
  );
  if (agentId) {
    console.log(`Message routed to agent: ${agentId}`);
  }
});

messageRouter.registerMessageHandler('agent-settings-response', (message) => {
  console.log('Received agent settings response:');
  message.agents.forEach((agent) => {
    console.log(`- ${agent.name} (${agent.id}): ${agent.status}`);
  });
});
```

### Registering Agents

```typescript
// Register an agent
const agent = {
  id: 'agent1',
  name: 'Test Agent',
  type: 'test',
  version: '1.0.0',
  settings: [
    {
      id: 'setting1',
      name: 'Test Setting',
      type: 'string',
      value: 'test value',
    },
  ],
  status: 'online',
};

agentRegistry.registerAgent(agent);
```

### Sending Messages

#### Text Message to All Agents

```typescript
const textMessage = AmpMessageFactory.createTextMessage(
  'Hello all agents',
  { type: 'all' },
  { id: 'owner1', name: 'Owner', type: 'owner' }
);

messageRouter.routeMessage(textMessage);
```

#### Text Message to Specific Agents

```typescript
const textMessage = AmpMessageFactory.createTextMessage(
  'Hello Agent 1',
  { type: 'specific', agentIds: ['agent1'] },
  { id: 'owner1', name: 'Owner', type: 'owner' }
);

messageRouter.routeMessage(textMessage);
```

#### File Message

```typescript
const fileMessage = AmpMessageFactory.createFileMessage(
  'test.txt',
  1024,
  'hash123',
  { type: 'all' },
  { id: 'owner1', name: 'Owner', type: 'owner' }
);

messageRouter.routeMessage(fileMessage);
```

#### Agent Settings Query

```typescript
const queryMessage = AmpMessageFactory.createAgentSettingsQuery(
  undefined, // Query all agents
  { id: 'owner1', name: 'Owner', type: 'owner' }
);

messageRouter.routeMessage(queryMessage);
```

## API

### Message Types

- `TextMessage`: Text message with content, target, and sender information
- `FileMessage`: File message with filename, size, hash, target, and sender information
- `AgentSettingsQuery`: Query for agent settings
- `AgentSettingsResponse`: Response with agent settings

### Classes

- `InMemoryAgentRegistry`: In-memory implementation of AgentRegistry interface
- `AmpMessageRouter`: Routes messages to appropriate agents
- `AmpMessageFactory`: Creates messages with proper structure

### Interfaces

- `AgentRegistry`: Interface for agent registration and retrieval
- `MessageRouter`: Interface for message routing
- `SenderInfo`: Information about the message sender
- `TargetInfo`: Information about the message target
- `AgentInfo`: Information about an agent
- `AgentSetting`: Setting for an agent

## Testing

```bash
pnpm test
```

## License

MIT
