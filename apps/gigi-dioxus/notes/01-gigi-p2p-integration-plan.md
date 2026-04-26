Okay, now we have a complete picture. Let's create a plan for integrating gigi-p2p into the gigi-dioxus project!

# Gigi-P2P Integration Plan for Dioxus Project

## Overview

The gigi-dioxus project already has the basic structure set up:
- Dependencies on `gigi-p2p`, `gigi-auth`, `gigi-store` are already in Cargo.toml
- Basic `P2pService` skeleton is in place
- Chat UI components are already implemented
- `MessageService` and chat state management are there (currently with mock data)

## Phase 1: Complete P2P Service and Event Handling

### Goals
- Connect P2P events to chat state updates
- Integrate gigi-store for persistence
- Complete missing P2P service methods

### Tasks
1. **Event Bus Implementation**
   - Use `futures-channel` (already a dependency) to create an event bus for P2P events
   - This allows UI components to subscribe to events

2. **Update P2pService.handle_event**
   - Handle all P2pEvent variants:
     - `PeerDiscovered`: Update peers list in chat state
     - `PeerConnected` / `PeerDisconnected`: Update peer online status
     - `DirectMessage`: Save to gigi-store, update UI
     - `GroupMessage`: Save to gigi-store, update UI
     - File sharing events: Download progress, completion
     - All other relevant events

3. **Integrate gigi-store**
   - Initialize MessageStore, ConversationStore, GroupManager, etc. from gigi-store
   - Connect persistence to P2pService
   - Update MessageService to use real gigi-store instead of mock data

4. **Complete P2pService Methods**
   - File sending/receiving methods
   - Group management methods (create, join, leave)
   - Peer management methods

## Phase 2: Connect UI to Real Data

### Goals
- Replace mock data in chat state with data from gigi-store
- Connect chat actions (send message, etc.) to real P2P methods

### Tasks
1. **Update Chat State Initialization**
   - On app load, fetch conversations, peers, groups from gigi-store
   - Update `use_chat_state` and related hooks

2. **Connect Chat Room to P2P**
   - Load message history from gigi-store for a chat room
   - Connect send message functionality to `P2pService::send_message`
   - Optimistic UI updates (show message immediately before it's confirmed)

3. **Implement Peer Discovery**
   - Show discovered peers in the UI
   - Handle peer connection status updates

4. **Group Chat Functionality**
   - Connect group creation/join/leave methods to UI
   - Show group messages in chat rooms

## Phase 3: File Sharing Implementation

### Goals
- Implement file sending/receiving
- Add thumbnail generation for images
- Show file messages in UI

### Tasks
1. **File Sending UI**
   - Implement file picker for selecting files to send
   - Show progress for file uploads

2. **File Receiving and Persistence**
   - Auto-download files or prompt user (follow patterns from current implementation)
   - Store files using gigi-store's FileSharingStore
   - Generate and store thumbnails for images

3. **File Message UI**
   - Show file messages with proper icons and thumbnails
   - Allow clicking files to open/download

## Phase 4: Complete UI Features

### Goals
- Add all missing chat features present in the current implementation
- Polish UI/UX

### Tasks
1. **Contact Management**
   - Add/remove contacts (using gigi-store's ContactManager)
   - Show contact list in "Me" tab

2. **Settings and Configuration**
   - Allow changing nickname, etc.
   - Store settings using gigi-store's SettingsManager

3. **Message Search**
   - Implement search messages functionality (use gigi-store's search methods)

4. **Error Handling and Loading States**
   - Properly handle P2P errors and show user-friendly messages
   - Add loading states for chat rooms, messages loading

## Architecture Reference (from existing projects)

### Event Flow (from gigi-dioxus)
```
P2pClient → P2pEvent → Event Bus → Chat State → UI Updates
           ↓
        Save to gigi-store
```

### State Management (from gigi-dioxus, adapted to Dioxus)
- Use Dioxus `use_signal` for chat state
- Use event bus for P2P events to update signals
- Persist state changes to gigi-store

### Key Files to Reference
- `/home/lebei/dev/gigi-dioxus/rust/gigi-p2p/examples/chat.rs`: Full CLI chat example
- `/home/lebei/dev/gigi-dioxus/apps/gigi-dioxus/src/services/`: Event handling and services
- `/home/lebei/dev/gigi-dioxus/rust/gigi-store/src/lib.rs`: Persistence layer