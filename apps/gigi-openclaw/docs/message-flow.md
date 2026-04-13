# OpenClaw Message Flow Analysis

## 1. How Channel Plugins Route Messages to Specific Agents

OpenClaw uses a multi-level routing mechanism to route messages from channels to specific agents:

1. **Binding Resolution**: In `src/channels/plugins/binding-routing.ts`, the `resolveConfiguredBindingRoute` function first attempts to resolve configured bindings to determine if there's a specific agent bound to the current session.

2. **Route Resolution**: In `src/routing/resolve-route.ts`, the `resolveAgentRoute` function determines the route based on the following priority:
   - `binding.peer`: Direct match to a specific peer
   - `binding.peer.parent`: Match to the parent thread's peer
   - `binding.peer.wildcard`: Match to wildcard peers
   - `binding.guild+roles`: Match based on Discord server roles
   - `binding.guild`: Match based on Discord server
   - `binding.team`: Match based on team
   - `binding.account`: Match based on account
   - `binding.channel`: Match based on channel
   - `default`: Use the default agent

3. **Session Key Generation**: A session key is generated based on the resolution result, which is used for subsequent message processing and state management.

## 2. How Agents Maintain Message History for LLM Query Context

Agents manage message history through the following process:

1. **Session History Tool**: In `src/agents/tools/sessions-history-tool.ts`, the `createSessionsHistoryTool` function creates a tool that allows agents to query session history.

2. **History Storage**: Message history is stored in session files and read through the `readSessionMessages` function.

3. **History Processing**:
   - Message history is processed by `sanitizeChatHistoryMessages` to remove sensitive information and oversized content
   - It is truncated by `truncateChatHistoryText` to fit within the LLM's context window
   - It is enhanced by `augmentChatHistoryWithCanvasBlocks` to add canvas content blocks

4. **Context Window Management**: The system automatically adjusts the number and size of historical messages based on the model's context window size to ensure it doesn't exceed the model's limits.

## 3. How Agents Respond to Messages Back to the Channel to the Agent Owner

Agents respond to messages through the following process:

1. **Message Preparation**: The agent generates a response message, which may include text, images, tool call results, etc.

2. **Message Broadcast**: In `src/gateway/server-methods/chat.ts`, the `broadcastChatFinal` function:
   - Assigns a sequence number to the message
   - Removes envelopes and inline directive tags from the message
   - Broadcasts the message to all connected clients
   - Sends the message to the specific session

3. **Delivery Routing**: The system determines the delivery route based on the message's source and destination to ensure the message is correctly sent back to the channel.

4. **Error Handling**: If an error occurs during the response process, the `broadcastChatError` function broadcasts an error message.

## 4. What Kinds of Message Types the Channel Supports

OpenClaw channels support multiple message types:

1. **Text Messages**: Basic text content with Markdown formatting support.

2. **Media Messages**:
   - Images: via base64 encoding or file paths
   - Audio: support for voice messages
   - Other media types: via media URLs or file paths

3. **Structured Messages**:
   - Tool calls: Agent requests to call tools
   - Tool results: Results of tool execution
   - Thinking messages: Agent's thought process (for transparent reasoning)
   - Canvas content: Visual content blocks

4. **Control Messages**:
   - Reply references: References to previous messages
   - Thread messages: Messages sent in specific threads
   - Status messages: Indicating agent status (e.g., typing)

5. **System Messages**:
   - Session metadata
   - Error messages
   - System notifications

Channel configurations can be adjusted through the `CommonChannelMessagingConfig` type, including message size limits, history limits, text chunking methods, and more.
