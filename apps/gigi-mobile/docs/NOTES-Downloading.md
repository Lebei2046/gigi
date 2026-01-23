## Summary: Why File Download Progress Bar Wasn't Showing

### Root Cause
Multiple React `memo` components had incorrect comparison functions that only checked message IDs, ignoring message content changes (like `isDownloading` and `downloadProgress`).

### Problem Chain

**1. FileMessageBubble Component (MessageBubble.tsx:49-51)**
```typescript
export default memo(MessageBubble, (prevProps, nextProps) => {
  return prevProps.message.id === nextProps.message.id  // ❌ Only checks ID!
})
```
- When Redux updated a message with new progress, the ID stayed the same
- `memo` thought props hadn't changed and skipped re-render
- Progress bar code existed but never executed

**2. MessageList Component (MessageList.tsx:74-79)**
```typescript
export default memo(MessageList, (prevProps, nextProps) => {
  return (
    prevProps.messages.length === nextProps.messages.length &&
    prevProps.messages.every((msg, i) => msg.id === nextProps.messages[i]?.id)  // ❌ Only checks IDs!
  )
})
```
- When any message's progress changed, the array IDs were unchanged
- `MessageList` itself wasn't re-rendering
- Child components never got new props

### The Fix
Updated both memo comparison functions to properly compare fields that affect rendering:

**MessageBubble comparison:**
```typescript
return (
  prev.id === next.id &&
  prev.isDownloading === next.isDownloading &&  // ✅ Added
  prev.downloadProgress === next.downloadProgress &&  // ✅ Added
  prev.content === next.content &&
  // ... other rendering fields
)
```

**MessageList comparison:**
```typescript
return prevProps.messages.every((prevMsg, i) => {
  const nextMsg = nextProps.messages[i]
  return (
    prevMsg.id === nextMsg.id &&
    prevMsg.isDownloading === nextMsg.isDownloading &&  // ✅ Added
    prevMsg.downloadProgress === nextMsg.downloadProgress &&  // ✅ Added
    prevMsg.content === nextMsg.content
  )
})
```

### Result
Now when download progress events arrive:
1. Backend emits `file-download-progress` events
2. Redux updates message with new `isDownloading` and `downloadProgress`
3. `MessageList` detects field changes → re-renders
4. `MessageBubble` receives new props → re-renders
5. `FileMessageBubble` shows/hides progress bar based on `isDownloading` flag