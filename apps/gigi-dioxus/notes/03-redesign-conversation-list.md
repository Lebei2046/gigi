I've successfully redesigned the conversation list in the chat feature. Here's what was implemented:

1. **Combined Group and Direct Chat Lists**: The conversation list now shows all conversations in a single, unified list instead of separate sections for groups and direct chats.

2. **Loaded Conversations from Persistence**: Added functionality to load conversations from the persistence layer, ensuring that the conversation history is preserved.

3. **Included Groups Not in Conversations**: Groups that don't have existing conversations are now appended to the conversation list, allowing users to see all their groups in one place.

4. **Handled Peer Discovery and Status Updates**: Updated the event listeners to properly handle peer discovery and status changes:
   - When a peer is discovered, it's added to the list if not already present, with online status set to true
   - When a peer comes online, their status is updated
   - When a peer goes offline, their status is updated

5. **Maintained Action Functionality**: For groups in the conversation list, users can still trigger "share" and "empty messages" actions. For peers, users can still trigger "empty messages" actions.

The code compiles successfully, but when building for WebAssembly, there are some network-related dependency issues with the P2P service, which is expected since P2P networking isn't fully supported in WebAssembly environments. However, the core functionality of the conversation list redesign has been implemented as requested.

The new conversation list provides a more streamlined user experience by showing all conversations and groups in a single list, with proper handling of peer status updates and discovery.
