**So, we've cleaned the code.  Now, let's check the flow of sending images/files on android again which works on desktop well.**


Let me trace through the Android file sending flow to understand how it works:
Now let me check the frontend side:
Now let me check how `sendFileMessageWithPath` is implemented in the TypeScript client:
Now let me check the `P2pClient::send_direct_file` to understand the full flow:
Now let me look at how file sharing is received and handled:
Let me trace the download/request flow:
Now let me create a comprehensive summary of the Android file sending flow. Here's the full trace:

## Android File/Image Sending Flow Analysis

### **Sending Side (Sender)**

**1. User Action:**
- User clicks "pick image" button in ChatRoom.tsx
- Calls `handleImageSelectWithDialog()` or `handleFileSelectWithDialog()`

**2. File Selection (Desktop vs Android):**
- **Desktop**: Uses Tauri's `dialog` plugin with `open()` → returns file path
- **Android**: Would use `@tauri-apps/plugin-android-fs` (not yet implemented in UI)

**3. Frontend (`ChatRoom.tsx`):**
- Calls `MessagingClient.selectImageFile()` or `selectAnyFile()`
- Gets file path (desktop) or content URI (Android)
- Creates optimistic message entry in Redux store immediately
- Calls `messaging_send_file_message_with_path` (direct) or `messaging_send_group_file_message_with_path` (group)

**4. Tauri Command (`lib.rs`):**
- `messaging_send_file_message_with_path` function receives file path
- **Check if content URI**: `file_path.starts_with("content://")`

**For Android Content URIs:**
1. Use `tauri_plugin_android_fs::AndroidFsExt` to open file
2. Read all data into buffer
3. Calculate base64 encoding of image
4. **Save to local storage**: 
   - Gets `download_folder` from config
   - Creates file: `gigi_share_{hash}.dat` in download directory
5. Call `client.send_direct_file(nickname, &saved_path)` 

**For Desktop (Regular paths):**
1. Read file directly with `fs::read()`
2. Calculate base64 encoding
3. Call `client.send_direct_file(nickname, &path)`

**5. P2pClient (`p2p_client.rs`):**
- `send_direct_file(nickname, file_path)`:
  1. Get peer_id from nickname via `peer_manager`
  2. **Call `file_manager.share_file(file_path)`** → returns share_code
  3. Get `SharedFile` from file_manager with share_code
  4. Detect file type using `mime_guess::from_path()`
  5. Send `DirectMessage::FileShare` to peer with:
     - `share_code`
     - `filename`
     - `file_size`
     - `file_type`

**6. FileSharingManager (`file_sharing.rs`):**
- `share_file(file_path)`:
  1. Canonicalize path (or use original for content URIs)
  2. Check file exists
  3. Get file metadata
  4. **Calculate BLAKE3 hash** of file content
  5. Check if already shared → if hash matches, return existing code
  6. Calculate chunk count (file_size / CHUNK_SIZE)
  7. Create `FileInfo` with hash and chunk_count
  8. **Save to `shared.json`** via `save_shared_files()`
  9. Store in memory HashMap with share_code as key
  10. Return share_code

**7. Return to Frontend:**
- Rust returns `messageId|base64Data`
- Frontend updates message with `imageData` data URL
- User sees image in chat immediately

---

### **Receiving Side (Receiver)**

**1. P2pClient receives `FileShareRequest` message:**
- Sent via libp2p's request-response protocol

**2. Event Handler (`event_handler.rs`):**
- Receives `FileSharingRequest::GetFileInfo(share_code)`
- Looks up file in `file_manager.shared_files` by share_code
- Sends `FileSharingResponse::FileInfo` back to requester

**3. Requester receives `FileInfo`:**
- Triggers `start_download_file()` in `DownloadManager`
- Creates temporary `.downloading` file
- Sends `FileSharingRequest::GetChunk` requests for each chunk

**4. Sender responds with chunks:**
- Reads chunk from disk
- Sends `FileSharingResponse::Chunk(data, hash)`

**5. Receiver assembles chunks:**
- Validates chunk hash
- Writes to temp file
- On completion:
  - Renames temp → final filename
  - Emits `FileDownloadCompleted` event

**6. Auto-download for Images:**
- `handle_p2p_event_with_fields()` in `lib.rs`
- Checks if `file_type.starts_with("image/")`
- **If true**: Automatically calls `client.download_file()`
- If false: Only emits event, waits for manual request

**7. Frontend receives events:**
- `file-download-started`
- `file-download-progress` (with percentage)
- `file-download-completed` with path
- `image-message-received` for auto-downloaded images

---

### **Key Points for Android:**

1. **Content URIs need `android-fs` plugin** to read files
2. **Content URIs must be saved to accessible storage** before sharing (because P2pClient's `share_file()` expects a regular file path)
3. **The download folder** (`/storage/emulated/0/Download/gigi`) is used for both:
   - Incoming downloaded files
   - Temporary storage of Android content URIs being shared
4. **`shared.json`** is also in download folder for persistence

The flow looks solid. The main Android-specific handling is:
- Reading content URIs via `android-fs` plugin
- Saving content URIs to local storage before P2P sharing
- Using `/storage/emulated/0/Download/gigi` as base directory