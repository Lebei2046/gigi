# Thumbnail Store Implementation

## Problem

Previously, thumbnails were stored in the `shared_files` table via `thumbnail_path`. However, this created a problem for P2P file sharing:

- **Alice** sends an image to **Bob**
- Alice generates a thumbnail and stores `{share_code → thumbnail_path}` in her `shared_files` table
- Bob receives the message with Alice's `share_code` and tries to get the thumbnail
- **Problem**: Bob's `shared_files` table doesn't have Alice's `share_code`, so `get_thumbnail_path()` fails
- **Result**: Bob sees only a placeholder instead of the actual image

## Solution: Thumbnail Store

We created a new `thumbnail_store` that maps **file paths to thumbnail paths**. This works because:

1. **Alice's side**: When she sends an image, she generates a thumbnail and stores `{source_file_path → thumbnail_path}`
2. **Bob's side**: When he downloads the image, he generates his own thumbnail and stores `{downloaded_file_path → thumbnail_path}`
3. Both sides now have their own thumbnail mappings, independent of each other

### Architecture

```
Alice's Computer                Bob's Computer
===================               ================
File: /home/alice/pic.jpg    File: /home/bob/downloads/pic.jpg
        ↓                               ↓
Thumbnail: thumb_abc.jpg         Thumbnail: thumb_xyz.jpg
        ↓                               ↓
thumbnail_store:                 thumbnail_store:
  /home/alice/pic.jpg             /home/bob/downloads/pic.jpg
    → thumb_abc.jpg                  → thumb_xyz.jpg
```

## Implementation Details

### Database Schema

New `thumbnails` table:

```sql
CREATE TABLE thumbnails (
    id INTEGER PRIMARY KEY,
    file_path TEXT UNIQUE,        -- Key: full file path (source or downloaded)
    thumbnail_path TEXT,          -- Value: thumbnail filename (not full path)
    created_at INTEGER
);
```

### Backend Changes

#### 1. New ThumbnailStore

```rust
pub struct ThumbnailStore {
    db: DatabaseConnection,
}

impl ThumbnailStore {
    // Store or update thumbnail mapping
    pub async fn store_thumbnail(&self, file_path: &str, thumbnail_path: &str) -> Result<()>;

    // Get thumbnail path by file path
    pub async fn get_thumbnail(&self, file_path: &str) -> Result<Option<String>>;

    // Delete thumbnail mapping
    pub async fn delete_thumbnail(&self, file_path: &str) -> Result<bool>;

    // Clean up old thumbnails
    pub async fn cleanup_old_thumbnails(&self, older_than_seconds: i64) -> Result<u64>;
}
```

#### 2. Updated get_file_thumbnail Command

**Before** (using share_code):
```rust
pub async fn get_file_thumbnail(share_code: String, ...) -> Result<String> {
    let thumbnail_path = file_sharing_store.get_thumbnail_path(&share_code).await?;
    // ... read and return thumbnail
}
```

**After** (using file_path):
```rust
pub async fn get_file_thumbnail(file_path: String, ...) -> Result<String> {
    let thumbnail_path = thumbnail_store.get_thumbnail(&file_path).await?;
    // ... read and return thumbnail
}
```

#### 3. Updated File Sending

When Alice sends an image:
1. Generate thumbnail
2. Store mapping: `{source_file_path → thumbnail_path}`
3. Send file message (without thumbnail path, since it's Alice's local path)

```rust
// In send_file_message_internal()
let thumbnail_path_str = if is_image {
    match thumbnail::generate_thumbnail(&actual_path, &thumbnail_dir, (200, 200), 70).await {
        Ok(thumbnail_filename) => {
            // Store in thumbnail_store
            let thumbnail_store = state.thumbnail_store.clone();
            tokio::spawn(async move {
                thumbnail_store.store_thumbnail(&actual_path_str, &thumbnail_filename).await
            });
            Some(full_thumbnail_path.to_string_lossy().to_string())
        }
        ...
    }
} else {
    None
};
```

#### 4. Updated File Download Completion

When Bob downloads an image:
1. Generate thumbnail
2. Store mapping: `{downloaded_file_path → thumbnail_path}`
3. Emit completion event

```rust
// In handle_file_download_completed()
let thumbnail_filename = generate_thumbnail_for_image(app_handle, path, share_code).await?;
if let Some(thumb) = thumbnail_filename {
    if let Some(thumbnail_path_only) = thumb.split('/').last() {
        let thumbnail_store = state.thumbnail_store.clone();
        tokio::spawn(async move {
            thumbnail_store.store_thumbnail(&file_path_str, &thumbnail_path_only).await
        });
    }
}
```

#### 5. Updated get_messages Command

Now returns `filePath` field so frontend can query thumbnails:

```rust
serde_json::json!({
    // ... other fields ...
    "filePath": file_path,  // New: local file path for thumbnail lookup
    "thumbnailPath": thumbnail_path,  // For new messages (optional)
})
```

### Frontend Changes

#### 1. Updated MessagingClient.getFileThumbnail()

**Before** (using shareCode):
```typescript
static async getFileThumbnail(shareCode: string): Promise<string> {
    return await GigiP2p.get_file_thumbnail({ shareCode })
}
```

**After** (using filePath):
```typescript
static async getFileThumbnail(filePath: string): Promise<string> {
    return await GigiP2p.get_file_thumbnail({ filePath })
}
```

#### 2. Updated ImageMessageBubble Component

Now uses `filePath` instead of `shareCode` to load thumbnails:

```typescript
if (message.filePath && !message.thumbnailData && !thumbnailData) {
    try {
        const data = await MessagingClient.getFileThumbnail(message.filePath)
        if (data && data.length > 0) {
            setThumbnailData(data)
        }
    } catch (error) {
        console.error('Failed to load thumbnail:', error)
    }
}
```

#### 3. Updated Redux Types

Added `filePath` to Message interface:

```typescript
export interface Message {
    // ... existing fields ...
    filePath?: string  // New: local file path for thumbnail lookup
}
```

#### 4. Updated loadThumbnailAsync Thunk

Now uses `filePath` as cache key instead of `shareCode`:

```typescript
export const loadThumbnailAsync = createAsyncThunk(
    'chatRoom/loadThumbnail',
    async (filePath: string, { rejectWithValue }) => {
        try {
            const thumbnail = await MessagingClient.getFileThumbnail(filePath)
            return { filePath, thumbnail }
        } catch (error) {
            return rejectWithValue({ filePath, error: String(error) })
        }
    }
)
```

## Benefits

1. **Decouples thumbnail storage from P2P sharing**: Each device maintains its own thumbnail cache
2. **Works with any file path**: Thumbnails are keyed by absolute file path, not share_code
3. **Simplifies message content**: Messages don't need to store thumbnail paths (which are device-specific)
4. **Consistent lookup**: Frontend always uses `filePath` to get thumbnails
5. **Better cache management**: Can clean up old thumbnails when files are deleted

## Migration Notes

The old `shared_files.thumbnail_path` column is still kept for backward compatibility with existing data, but new code uses the `thumbnail_store` instead.

## Testing

To verify the implementation:

1. **Alice** sends an image to Bob
2. **Bob** should see the thumbnail immediately (not a placeholder)
3. **Bob** closes and reopens the chat - thumbnail should still be visible
4. **Bob** sends an image back to Alice
5. **Alice** should see Bob's thumbnail
6. Check that both Alice and Bob have their own thumbnail files in `download_folder/thumbnails/`
