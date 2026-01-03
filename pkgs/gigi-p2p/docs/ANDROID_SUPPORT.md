**So, summarize the solution to support android in gigi-p2p**


## Summary: Supporting Android Content URIs in gigi-p2p

### 1. Replace `SharedFile.path` with `FilePath` enum

**File: `pkgs/gigi-p2p/src/events.rs`**

```rust
use url::Url;

pub enum FilePath {
    Url(Url),      // Android content:// or iOS file:// URIs
    Path(PathBuf),  // Regular filesystem paths
}

pub struct SharedFile {
    pub id: String,
    pub info: FileInfo,
    pub path: FilePath,  // Changed from PathBuf
    // ... other fields
}
```

### 2. Add callback for reading file chunks

**File: `pkgs/gigi-p2p/src/client/event_handler.rs`**

```rust
type FileChunkReader = Arc<dyn Fn(&FilePath, u64, usize) -> Result<Vec<u8>> + Send + Sync>;

pub struct FileSharingManager<R> {
    // ... existing fields
    chunk_reader: Option<FileChunkReader>,
}

impl<R> FileSharingManager<R> {
    pub fn set_chunk_reader(&mut self, reader: FileChunkReader) {
        self.chunk_reader = Some(reader);
    }
}
```

### 3. Update chunk reading in DownloadManager

**File: `pkgs/gigi-p2p/src/client/download_manager.rs`**

```rust
impl DownloadManager {
    pub fn read_chunk(&self, sharing_code: &str, chunk_index: usize) -> Result<Vec<u8>> {
        let file = self.get_shared_file(sharing_code)?;
        let offset = (chunk_index * CHUNK_SIZE) as u64;
        
        match &file.path {
            FilePath::Path(path) => {
                // Regular file - use std::fs
                let mut file = std::fs::File::open(path)?;
                file.seek(SeekFrom::Start(offset))?;
                let mut buffer = vec![0u8; CHUNK_SIZE];
                let bytes_read = file.read(&mut buffer)?;
                buffer.truncate(bytes_read);
                Ok(buffer)
            }
            FilePath::Url(url) => {
                // Content URI or file:// URI - use callback
                if let Some(reader) = &self.chunk_reader {
                    reader(&file.path, offset, CHUNK_SIZE)
                } else {
                    Err(Error::msg("No chunk reader configured for URIs"))
                }
            }
        }
    }
}
```

### 4. Tauri app layer implementation

**File: `apps/gigi-mobile/src-tauri/src/lib.rs`**

```rust
use tauri_plugin_android_fs::AndroidFsExt;
use tauri_plugin_fs::FilePath as TauriFilePath;

#[cfg(mobile)]
fn setup_file_sharing(app_handle: &AppHandle) -> Result<()> {
    let chunk_reader = Arc::new({
        let app_handle = app_handle.clone();
        move |file_path: &gigi_p2p::FilePath, offset: u64, size: usize| -> Result<Vec<u8>> {
            match file_path {
                gigi_p2p::FilePath::Path(path) => {
                    // Regular file path
                    let mut file = std::fs::File::open(path)?;
                    file.seek(SeekFrom::Start(offset))?;
                    let mut buffer = vec![0u8; size];
                    let bytes_read = file.read(&mut buffer)?;
                    buffer.truncate(bytes_read);
                    Ok(buffer)
                }
                gigi_p2p::FilePath::Url(url) => {
                    // Use tauri_plugin_android_fs for content URIs
                    let uri = FileUri::from_str(url.as_str())?;
                    let mut file = app_handle.android_fs().open_file_readable(&uri).await?;
                    
                    file.seek(SeekFrom::Start(offset))?;
                    let mut buffer = vec![0u8; size];
                    let bytes_read = file.read(&mut buffer)?;
                    buffer.truncate(bytes_read);
                    Ok(buffer)
                }
            }
        }
    });
    
    // Pass to gigi-p2p
    app_handle.file_sharing_manager().set_chunk_reader(chunk_reader);
    
    Ok(())
}
```

### 5. Update file sharing to accept FilePath

**File: `pkgs/gigi-p2p/src/client/file_sharing.rs`**

```rust
impl<R> FileSharingManager<R> {
    // Existing method for regular files
    pub fn share_file(&mut self, path: &Path) -> Result<SharedFile> {
        let path = path.to_path_buf();
        if !path.exists() {
            return Err(Error::msg("File does not exist"));
        }
        // ... existing implementation
    }
    
    // New method for content URIs
    pub fn share_content_uri(&mut self, uri: &str, name: &str, size: u64) -> Result<SharedFile> {
        let url = Url::parse(uri)?;
        let sharing_code = generate_sharing_code();
        
        let shared_file = SharedFile {
            id: Uuid::new_v4().to_string(),
            info: FileInfo {
                name: name.to_string(),
                size,
                mime_type: mime_guess::from_path(name).first_or_octet_stream(),
                created_at: SystemTime::now(),
            },
            path: FilePath::Url(url),
            sharing_code: sharing_code.clone(),
            // ... other fields
        };
        
        // No file.exists() check needed for URIs
        self.files.insert(sharing_code, shared_file.clone());
        Ok(shared_file)
    }
}
```

### 6. Platform-specific behavior

| Platform | FilePath::Url | FilePath::Path | Reading Method |
|----------|---------------|-----------------|----------------|
| **Android** | `content://...` | Regular paths | `tauri_plugin_android_fs::open_file_readable()` |
| **iOS** | `file://...` | Regular paths | `std::fs::File::open()` (URL → path) |
| **Desktop** | N/A | Regular paths | `std::fs::File::open()` |

### Key Points

1. **gigi-p2p remains pure** - No Tauri dependencies, uses callback pattern
2. **Callback handles platform specifics** - Tauri app layer uses `tauri_plugin_android_fs`
3. **Efficient chunk reading** - No need to load entire file into memory
4. **Type-safe enum** - `FilePath` cleanly separates URIs from paths
5. **Backward compatible** - Existing `share_file(&Path)` method preserved

### Dependencies

Add to `pkgs/gigi-p2p/Cargo.toml`:
```toml
[dependencies]
url = "2"
```

Add to `apps/gigi-mobile/src-tauri/Cargo.toml`:
```toml
[dependencies]
tauri-plugin-android-fs = "24"
```