# Message Design Summary: Claw Owner and Agents Communication

## Overview
The communication system enables the Claw Owner (OpenClaw node owner) to interact with their agents through the Gigi P2P network. All messages include sender information to ensure proper identification.

## Message Types and Structures

### 1. Text Message
Used for sending text content between owner and agents.

**Structure:**
```json
{
  "type": "text",
  "content": "Hello, this is a text message",
  "target": "owner", // Can be: single agent ID, array of agent IDs, "owner", or "all"
  "sender": {
    "id": "agent1",
    "name": "Agent One",
    "type": "agent" // Can be "agent" or "owner"
  }
}
```

### 2. File Share Message
Used for sharing files between owner and agents.

**Structure:**
```json
{
  "type": "fileShare",
  "shareCode": "abc123",
  "filename": "document.pdf",
  "fileSize": 1024,
  "fileType": "application/pdf",
  "target": "owner", // Can be: single agent ID, array of agent IDs, "owner", or "all"
  "sender": {
    "id": "agent2",
    "name": "Agent Two",
    "type": "agent"
  }
}
```

### 3. Agent Settings Query
Used by the Claw Owner to request agent settings.

**Structure:**
```json
{
  "type": "agentSettingsQuery",
  "targetAgents": ["agent1"], // Can be: single agent ID, array of agent IDs, or "all"
  "sender": {
    "id": "owner",
    "name": "Claw Owner",
    "type": "owner"
  }
}
```

### 4. Agent Settings Response
Used by the OpenClaw node to return agent settings to the Claw Owner.

**Structure:**
```json
{
  "type": "agentSettingsResponse",
  "settings": [
    {
      "agentId": "agent1",
      "name": "Agent One",
      "status": "online",
      "capabilities": ["chat", "file-processing"],
      "config": {
        "timeout": 30000,
        "maxRetries": 3
      }
    }
  ],
  "sender": {
    "id": "node",
    "name": "OpenClaw Node",
    "type": "node"
  }
}
```

## Communication Flows

### 1. Agent → Owner
- **Text Message:** Agent sends text message with `target: "owner"` and includes sender information
- **File Share:** Agent sends file share message with `target: "owner"` and includes sender information

### 2. Owner → Agent(s)
- **Text Message:** Owner sends text message with `target` set to specific agent ID(s) or "all", including sender information
- **File Share:** Owner sends file share message with `target` set to specific agent ID(s) or "all", including sender information

### 3. Owner Queries Agent Settings
- **Query:** Owner sends `agentSettingsQuery` with `targetAgents` set to specific agent ID(s) or "all"
- **Response:** OpenClaw node sends `agentSettingsResponse` with settings for requested agents, including sender information

## Routing Logic

1. **Message Reception:** Gigi plugin receives message from P2P network
2. **Type Identification:** Determines message type (text, fileShare, agentSettingsQuery)
3. **Sender Verification:** Validates sender information
4. **Target Processing:**
   - If `target` is "owner": Route to Claw Owner
   - If `target` is specific agent ID(s): Route to those agents
   - If `target` is "all": Route to all agents
5. **Special Handling:**
   - For `agentSettingsQuery`: Collect settings and send `agentSettingsResponse`
   - For `fileShare`: Handle file transfer using share code

## Key Features

- **Sender Identification:** All messages include sender information, resolving the bug where the owner couldn't identify which agent sent messages
- **Flexible Targeting:** Messages can be sent to specific agents, multiple agents, or all agents
- **Structured Settings Queries:** Owner can request and receive detailed agent settings
- **Backward Compatibility:** Maintains support for existing message types
- **Secure:** Leverages Gigi P2P network's encryption and peer verification

This design provides a robust framework for communication between the Claw Owner and agents, ensuring clear identification of message senders while enabling flexible targeting and settings management.