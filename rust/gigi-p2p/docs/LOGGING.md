# Logging Best Practices for Library Development

This document outlines the logging best practices implemented in `gigi-p2p` library.

## 🏗️ Core Principles

### 1. Library-Friendly Design
```rust
// ✅ Good: Provide convenience functions but don't force initialization
pub fn init_tracing() { /* ... */ }

// ❌ Avoid: Don't auto-initialize logging in library constructors
pub fn new() -> Self {
    // init_tracing(); // Don't do this
}
```

### 2. Structured Logging
```rust
// ✅ Good: Use structured fields
info!(
    file_id = %share_code,
    filename = %file_name,
    size = %metadata.len(),
    "Shared file successfully"
);

// ❌ Avoid: String interpolation
info!("Shared file {} with size {}", file_name, metadata.len());
```

### 3. Appropriate Log Levels
```rust
// ERROR: When the library cannot continue
error!("Failed to initialize P2P client: {}", e);

// WARN: Recoverable situations
warn!("Peer {} already connected, skipping", peer_id);

// INFO: High-level operations users care about
info!("File '{}' shared with code: {}", filename, share_code);

// DEBUG: Detailed flow information  
debug!("Processing swarm event: {:?}", event);

// TRACE: Very detailed execution flow
trace!("Sending request to peer: {}", peer_id);
```

## 📝 Implementation Patterns

### 1. Function Instrumentation
```rust
#[instrument(skip(self))]
pub fn join_group(&mut self, group_name: &str) -> Result<()> {
    info!("Joining group: {}", group_name);
    // Function implementation
}

#[instrument(skip(self, message))]
pub async fn send_group_message(&mut self, group_name: &str, message: String) -> Result<()> {
    // Function implementation
}
```

### 2. Error Handling
```rust
fn send_event(&self, event: P2pEvent) {
    if let Err(e) = self.event_sender.unbounded_send(event) {
        error!(
            error = %e,
            event_type = %std::mem::discriminant(&event),
            "Failed to send P2P event"
        );
    }
}
```

### 3. Context Preservation
```rust
#[instrument(skip(self), fields(group_name = group_name))]
pub async fn send_group_message(&mut self, group_name: &str, message: String) -> Result<()> {
    // Fields are automatically included in all log statements within this function
    info!("Sending group message to: {}", group_name);
    // ...
}
```

## 🔧 Configuration

### 1. Consumer Control
```rust
// Consumers can choose their own logging setup
use tracing_subscriber;

// Custom setup for production
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::WARN)
    .with_target(false)
    .json()  // Structured logs for production
    .init();

// Or use convenience function for development
gigi_p2p::init_tracing();
```

### 2. Environment Variables
```bash
# Set log level via environment
RUST_LOG=info
RUST_LOG=gigi_p2p=debug
RUST_LOG=debug,gigi_p2p=trace
```

## 📚 Documentation Standards

### 1. API Documentation
```rust
/// Share a file with other peers
/// 
/// # Logging
/// - INFO: Logs successful file sharing with share code
/// - ERROR: Logs file access or hashing failures
/// - DEBUG: Logs internal processing steps when enabled
/// 
/// # Example
/// ```rust
/// // Enable logging
/// gigi_p2p::init_tracing();
/// 
/// let share_code = client.share_file(Path::new("document.pdf")).await?;
/// ```
#[instrument(skip(self))]
pub async fn share_file(&mut self, path: &Path) -> Result<String, P2pError> {
    // Implementation
}
```

### 2. Runtime Log Examples
```
INFO  gigi_p2p::client: Shared file 'document.pdf' (hash: a1b2c3d4) with code: abc123
DEBUG gigi_p2p::client: Found peer: 12D3KooWBM7G7WrZqHqKXj1yEHY at /ip4/192.168.1.100/tcp/7654
ERROR gigi_p2p::client: Failed to send P2P event: channel closed
```

## 🎯 Performance Considerations

### 1. Lazy Evaluation
```rust
// ✅ Good: Expensive operations only when needed
debug!("Hash: {:?}", Blake3::hash(&data));

// ❌ Avoid: Always computing expensive values
let hash = Blake3::hash(&data);
debug!("Computed hash: {:?}", hash);
```

### 2. Skip Large Data
```rust
// ✅ Good: Skip large data structures
#[instrument(skip(self, large_data))]
fn process_large_data(&self, large_data: &[u8]) {
    // Implementation
}
```

### 3. Use Display Instead of Debug
```rust
// ✅ Good: Use Display for user-facing logs
info!("File '{}' shared", filename);

// ❌ Avoid: Debug for user-facing logs  
info!("File '{:?}' shared", filename);
```

## 🚀 Advanced Patterns

### 1. Filtered Logging
```rust
// Log different levels for different modules
use tracing_subscriber::{fmt, EnvFilter};

fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

### 2. Custom Targets
```rust
// Use custom targets for better filtering
info!(target: "gigi_p2p::file_transfer", "File operation completed");
```

### 3. Span Context
```rust
// Create spans for complex operations
#[instrument(skip(self))]
async fn download_file(&mut self, share_code: &str) -> Result<String, P2pError> {
    let _span = info_span!("file_download", share_code = share_code);
    let _enter = _span.enter();
    
    info!("Starting file download");
    // Download implementation
    info!("File download completed");
}
```

## 📊 Monitoring Integration

### 1. Metrics Integration
```rust
// Combine logging with metrics
use tracing_subscriber::{fmt, prelude::*};

fmt()
    .with_span_events(fmt::format::FmtSpan::CLOSE)
    .with_filter(filter::EnvFilter::from_default_env())
    .init();
```

### 2. Structured Output
```rust
// JSON format for log aggregation
fmt()
    .json()
    .with_current_span(true)
    .with_span_list(true)
    .init();
```

These practices ensure that:
- Consumers have full control over logging configuration
- Library provides useful debugging information when needed
- Performance impact is minimal when logging is disabled
- Logs are structured and searchable
- Different environments can use appropriate log levels