# @gigi/message-types

Shared message type definitions for the Gigi P2P ecosystem.

## Overview

`@gigi/message-types` provides a set of TypeScript interfaces and types for defining messages in the Gigi P2P ecosystem. These types are used across various components to ensure consistent message formatting and validation.

## Features

- **Comprehensive Message Types**: Includes definitions for text messages, file messages, group shares, and file shares
- **Type Safety**: TypeScript interfaces for strict type checking
- **Type Guards**: Utility functions to check message types at runtime
- **Shared Interfaces**: Consistent message structure across all Gigi components

## Installation

```bash
pnpm add @gigi/message-types
```

## Usage

### Importing Types

```typescript
import {
  GigiMessage,
  TextMessage,
  FileMessage,
  GroupShareMessage,
  FileShareMessage,
  MessageContent,
  MessageContentInput,
  isTextMessage,
  isFileMessage,
  isGroupShareMessage,
  isFileShareMessage
} from '@gigi/message-types';
```

### Creating a Text Message

```typescript
import { TextMessage } from '@gigi/message-types';

const textMessage: TextMessage = {
  type: 'text',
  content: 'Hello, Gigi!',
  target: {
    type: 'specific',
    agentIds: ['main']
  },
  sender: {
    id: 'user123',
    name: 'User',
    type: 'owner'
  },
  timestamp: Date.now(),
  id: 'msg-123'
};
```

### Using Type Guards

```typescript
import { GigiMessage, isTextMessage, isFileMessage } from '@gigi/message-types';

function handleMessage(message: GigiMessage) {
  if (isTextMessage(message)) {
    // Handle text message
    console.log('Text message:', message.content);
  } else if (isFileMessage(message)) {
    // Handle file message
    console.log('File message:', message.filename);
  }
}
```

### Message Content for P2P Client

```typescript
import { MessageContent, MessageContentInput } from '@gigi/message-types';

// Input for sending a message
const messageInput: MessageContentInput = {
  type: 'text',
  text: 'Hello from P2P client'
};

// Received message
const receivedMessage: MessageContent = {
  type: 'text',
  text: 'Hello back!',
  fromPeerId: 'peer123',
  fromNickname: 'Peer'
};
```

## Message Types

### TextMessage

Represents a text message with content, target, and sender information.

```typescript
interface TextMessage {
  type: 'text';
  content: string;
  target: TargetInfo;
  sender: SenderInfo;
  timestamp: number;
  id: string;
}
```

### FileMessage

Represents a file message with file metadata, target, and sender information.

```typescript
interface FileMessage {
  type: 'file';
  filename: string;
  fileSize: number;
  fileHash: string;
  target: TargetInfo;
  sender: SenderInfo;
  timestamp: number;
  id: string;
}
```

### GroupShareMessage

Represents a message for sharing group information with other peers.

```typescript
interface GroupShareMessage {
  type: 'shareGroup';
  groupId: string;
  groupName: string;
  inviterNickname: string;
  target: TargetInfo;
  sender: SenderInfo;
  timestamp: number;
  id: string;
}
```

### FileShareMessage

Represents a message for sharing files between peers.

```typescript
interface FileShareMessage {
  type: 'fileShare';
  shareCode: string;
  filename: string;
  fileSize: number;
  fileType: string;
  target: TargetInfo;
  sender: SenderInfo;
  timestamp: number;
  id: string;
}
```

## Supporting Types

### SenderInfo

Information about the message sender.

```typescript
interface SenderInfo {
  id: string;
  name: string;
  type: 'owner' | 'agent' | 'node';
  nodeId?: string; // Optional node ID for agent senders
}
```

### TargetInfo

Information about the message target for routing.

```typescript
interface TargetInfo {
  type: 'all' | 'specific' | 'node' | 'node-agent';
  agentIds?: string[];
  nodeId?: string; // Required for node and node-agent types
}
```

### MessageContent

Message content types used by the P2P client.

```typescript
type MessageContent =
  | { type: 'text'; text: string; fromPeerId: string; fromNickname: string }
  | { type: 'fileShare'; shareCode: string; filename: string; fileSize: number; fileType: string; fromPeerId: string; fromNickname: string }
  | { type: 'shareGroup'; groupId: string; groupName: string; inviterNickname: string; fromPeerId: string; fromNickname: string };
```

### MessageContentInput

Input type for sending messages (without auto-populated fields).

```typescript
type MessageContentInput =
  | { type: 'text'; text: string }
  | { type: 'fileShare'; shareCode: string; filename: string; fileSize: number; fileType: string }
  | { type: 'shareGroup'; groupId: string; groupName: string; inviterNickname: string };
```

## Type Guards

### isTextMessage(message: any): message is TextMessage

Checks if a message is a text message.

### isFileMessage(message: any): message is FileMessage

Checks if a message is a file message.

### isGroupShareMessage(message: any): message is GroupShareMessage

Checks if a message is a group share message.

### isFileShareMessage(message: any): message is FileShareMessage

Checks if a message is a file share message.

## Testing

```bash
# Run tests
pnpm test
```

## Linting and Formatting

```bash
# Lint code
pnpm lint

# Fix linting issues
pnpm lint:fix

# Format code
pnpm format
```
