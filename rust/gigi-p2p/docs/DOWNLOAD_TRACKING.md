# Mobile App Download Tracking Integration

This document explains how to integrate the enhanced download tracking system in `gigi-mobile` for displaying file downloads with proper UI feedback.

## 🎯 Problem Solved

For mobile apps like `gigi-mobile`, when a user downloads a file, the UI needs to:
1. Track download progress with filenames and share codes
2. Display real-time progress indicators
3. Show completion/error states
4. Allow retrying failed downloads
5. Display downloaded images/files immediately after completion

## 🔄 file_id vs download_id

### Key Distinction:
- **file_id**: Unique identifier for the shared file content (same for all downloads of that file)
- **download_id**: Unique identifier for each specific download instance (different for Bob and Kelvin downloading the same file)

### Example Scenario:
```
Alice shares photo.jpg → file_id: "abc123"
Bob downloads photo.jpg → download_id: "dl_abc123_peerBob_1234567890"
Kelvin downloads photo.jpg → download_id: "dl_abc123_peerKelvin_1234567891"
```

This enables:
- Multiple simultaneous downloads of the same file
- Individual progress tracking per downloader
- Proper UI state management when peers download identical files

## 🔧 Solution: Enhanced Download Tracking

### New Events with Rich Context

All download events now include:
- `filename` - Human-readable filename
- `share_code` - Share code for reference
- `from_nickname` - Sender's nickname

```rust
// Enhanced events
FileDownloadProgress {
    download_id: String,     // ⬅️ UNIQUE PER DOWNLOAD INSTANCE
    filename: String,        // ⬅️ NEW
    share_code: String,      // ⬅️ NEW  
    from_nickname: String,   // ⬅️ NEW
    downloaded_chunks: usize,
    total_chunks: usize,
}

FileDownloadCompleted {
    download_id: String,     // ⬅️ UNIQUE PER DOWNLOAD INSTANCE
    filename: String,        // ⬅️ NEW
    share_code: String,      // ⬅️ NEW
    from_nickname: String,   // ⬅️ NEW
    path: PathBuf,
}

FileDownloadFailed {
    download_id: String,     // ⬅️ UNIQUE PER DOWNLOAD INSTANCE
    filename: String,        // ⬅️ NEW
    share_code: String,      // ⬅️ NEW
    from_nickname: String,   // ⬅️ NEW
    error: String,
}
```

### ActiveDownload Struct for UI State

```rust
pub struct ActiveDownload {
    pub download_id: String, // ⬅️ UNIQUE PER DOWNLOAD INSTANCE
    pub filename: String,
    pub share_code: String,
    pub from_peer_id: PeerId,
    pub from_nickname: String,
    pub total_chunks: usize,
    pub downloaded_chunks: usize,
    pub started_at: Instant,
    pub completed: bool,
    pub failed: bool,
    pub error_message: Option<String>,
    pub final_path: Option<PathBuf>,
}
```

## 📱 Mobile App Integration

### 1. Track Downloads in State

```dart
// In gigi-mobile state management
class DownloadState {
  final Map<String, ActiveDownload> activeDownloads = {};
  
  void handleEvent(P2pEvent event) {
    switch (event.type) {
      case 'FileDownloadStarted':
        // Show download started UI
        break;
      case 'FileDownloadProgress':
        // Update progress bar
        final download = activeDownloads[event.downloadId];
        if (download != null) {
          download.downloadedChunks = event.downloadedChunks;
          download.totalChunks = event.totalChunks;
          
          // Update UI with progress percentage
          final progress = (event.downloadedChunks / event.totalChunks) * 100;
          updateDownloadProgress(event.downloadId, progress);
        }
        break;
      case 'FileDownloadCompleted':
        // Mark as completed, show image/file
        final download = activeDownloads[event.downloadId];
        if (download != null) {
          download.completed = true;
          download.finalPath = event.path;
          
          // If it's an image, display it immediately
          if (isImageFile(event.filename)) {
            showDownloadedImage(event.filename, event.path);
          }
        }
        break;
      case 'FileDownloadFailed':
        // Show error with retry option
        final download = activeDownloads[event.downloadId];
        if (download != null) {
          download.failed = true;
          download.errorMessage = event.error;
          
          showDownloadError(event.filename, event.error, () {
            // Retry download
            client.download_file(event.fromNickname, event.shareCode);
          });
        }
        break;
    }
  }
}
```

### 2. Query Active Downloads

```dart
// Get all active downloads for UI display
List<ActiveDownload> getAllDownloads() {
  return client.get_active_downloads();
}

// Get downloads from specific peer
List<ActiveDownload> getDownloadsFromPeer(String nickname) {
  return client.get_downloads_from_peer(nickname);
}

// Get recent downloads for history
List<ActiveDownload> getRecentDownloads() {
  return client.get_recent_downloads(20);
}

// Cleanup completed downloads
void cleanupDownloads() {
  client.cleanup_downloads();
}
```

### 3. UI Components

#### Download Progress Indicator
```dart
Widget DownloadProgressCard(ActiveDownload download) {
  final progress = download.totalChunks > 0 
      ? (download.downloadedChunks / download.totalChunks) * 100 
      : 0.0;
      
  return Card(
    child: Column(
      children: [
        Text(download.filename),
        LinearProgressIndicator(value: progress / 100),
        Text('${progress.toStringAsFixed(1)}%'),
        if (download.completed) 
          Icon(Icons.check_circle, color: Colors.green)
        else if (download.failed)
          Icon(Icons.error, color: Colors.red)
      ],
    ),
  );
}
```

#### Image Display After Download
```dart
Widget DownloadedImage(String filename, String filePath) {
  if (isImageFile(filename)) {
    return Image.file(File(filePath));
  } else {
    return FileIcon(filename);
  }
}
```

## 🔄 Typical Flow

### User Downloads an Image:

1. **User Action**: Tap "Download" in chat
2. **Download Started**: `FileDownloadStarted` event → Show progress card
3. **Progress Updates**: `FileDownloadProgress` events → Update progress bar
4. **Download Completed**: `FileDownloadCompleted` event → 
   - Mark as completed ✅
   - If image: Display immediately in chat
   - If file: Show file icon with open option
5. **UI Update**: Remove from active downloads, move to history

### Error Handling:

1. **Download Failed**: `FileDownloadFailed` event → 
   - Show error message
   - Display retry button
   - Keep in history for reference

## 🎨 UI Benefits

- **Real-time Progress**: Live percentage updates for all downloads
- **Visual Feedback**: Clear completed/failed states
- **Image Preview**: Images show immediately after download
- **Retry Functionality**: Easy retry for failed downloads
- **History**: Recent downloads accessible in chat history
- **Peer Context**: Shows who sent each file

## 📋 Implementation Checklist

For `gigi-mobile`:

- [ ] Update P2P event handlers to use new event fields
- [ ] Add `ActiveDownload` tracking to app state
- [ ] Implement download progress UI components
- [ ] Add image display after download completion
- [ ] Implement error handling with retry
- [ ] Add download history section
- [ ] Update TypeScript/Flutter types for new events
- [ ] Test with various file types (images, documents)
- [ ] Test error scenarios (network issues, hash mismatches)

## 🚀 Result

Users will now see:
- ⬇️ "Downloading image.jpg from Alice..."
- 📊 "45% complete" with progress bar
- ✅ "Download complete - Click to view" 
- 🖼️ Image displayed directly in chat
- 🔄 "Retry" button if download fails

This provides the polished mobile experience users expect! 🎉