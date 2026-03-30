# Gigi OpenClaw Plugin Test Plan

## Overview

This test plan outlines the steps to test the integration between the Gigi OpenClaw Plugin and the Gigi P2P Example client. The goal is to verify that both clients can chat and share files in a group.

## Prerequisites

- Node.js 18+ or compatible runtime
- pnpm (recommended for monorepo setup)
- OpenClaw 2026.3.22+
- A test file for file sharing (e.g., `test-file.txt`)

## Test Setup

### 1. Build the Projects

```bash
# Build the entire Gigi monorepo
cd /home/lebei/dev/gigi
pnpm install

# Build gigi-p2p-ts (core P2P functionality)
cd typescript/gigi-p2p-ts
pnpm build

# Build gigi-openclaw plugin
cd ../../apps/gigi-openclaw
pnpm run build:bundle

# Build gigi-p2p-example
cd ../../typescript/gigi-p2p-example
pnpm install
```

### 2. Install or Update the Gigi OpenClaw Plugin

```bash
# Install or update the plugin in OpenClaw
openclaw plugins install /home/lebei/dev/gigi/apps/gigi-openclaw

# Check if the plugin is loaded
openclaw plugins list

# You should see "Gigi P2P" with status "loaded"
```

### 3. Configure OpenClaw with Gigi P2P

1. Start OpenClaw gateway:
   ```bash
   openclaw gateway start
   ```

2. Generate a mnemonic for consistent peer ID (if not already configured):
   ```bash
   cd /home/lebei/dev/gigi/apps/gigi-openclaw
   pnpm run generate-mnemonic
   ```

3. Update the channel configuration with the mnemonic and peer ID:
   ```bash
   openclaw config set channels.gigi-p2p-bundled.mnemonic "your-mnemonic-here"
   openclaw config set channels.gigi-p2p-bundled.peerId "your-peer-id-here"
   openclaw config set channels.gigi-p2p-bundled.enabled true
   ```

4. Restart the gateway to apply the configuration:
   ```bash
   openclaw gateway restart
   ```

## Test Cases

### Test Case 1: Basic Group Chat

#### Steps
1. **Start gigi-p2p-example client:**
   ```bash
   cd /home/lebei/dev/gigi/typescript/gigi-p2p-example
   npx tsx index.ts "ExampleClient"
   ```

2. **In the example client, join a group:**
   ```
   /join test-group
   ```

3. **In OpenClaw, join the same group and send a message:**
   ```bash
   openclaw message send --channel gigi-p2p-bundled --target "test-group" --message "Hello from OpenClaw!"
   ```

4. **Verify message is received in both clients:**
   - Example client should show the message from OpenClaw
   - OpenClaw should show the message in its logs

### Test Case 2: File Sharing in Group

#### Steps
1. **In gigi-p2p-example client, share a file:**
   ```
   /share /path/to/test-file.txt
   ```
   Note the share code returned.

2. **In gigi-p2p-example client, share the file in the group:**
   ```
   /group test-group /file <share-code>
   ```

3. **Verify OpenClaw receives the file share message:**
   - OpenClaw logs should show the file share notification

4. **In OpenClaw, download the file:**
   - Note: The download functionality is handled automatically when OpenClaw receives a file share message
   - Check the OpenClaw logs for download progress
   - Verify the file is downloaded to the configured downloads directory

5. **Verify the file is downloaded successfully:**
   - Check the downloads directory for the file

### Test Case 3: Direct File Sharing

#### Steps
1. **In OpenClaw, share a file:**
   - Create a test file with some content
   - Use the `sendMedia` functionality to share the file:
   ```bash
   openclaw message send --channel gigi-p2p-bundled --target "<example-client-peer-id>" --media "/path/to/test-file.txt"
   ```
   - Check the OpenClaw logs for the share code

2. **In gigi-p2p-example client, download the file:**
   ```
   /download <share-code>
   ```

3. **Verify the file is downloaded successfully:**
   - Check the downloads directory for the file

### Test Case 4: Multiple Group Membership

#### Steps
1. **In gigi-p2p-example client, join another group:**
   ```
   /join another-group
   ```

2. **In OpenClaw, join the same group:**
   ```bash
   openclaw message send --channel gigi-p2p-bundled --target "another-group" --message "Hello from OpenClaw in another group!"
   ```

3. **Verify messages are received in the correct groups:**
   - Messages sent to `test-group` should only appear in that group
   - Messages sent to `another-group` should only appear in that group

## Verification

### Success Criteria

1. **Group Chat:** Both clients can send and receive messages in the same group
2. **File Sharing:** Both clients can share and download files using share codes
3. **Multiple Groups:** Both clients can join multiple groups and communicate in each
4. **Error Handling:** Both clients handle errors gracefully (e.g., invalid share codes)

### Logs and Debugging

- **Gigi OpenClaw Plugin logs:** Check OpenClaw logs for plugin activity
  ```bash
  openclaw logs
  ```

- **Gigi P2P Example logs:** Check the console output of the example client

- **Network Debugging:** Use `libp2p-debug` to monitor P2P traffic if needed

## Cleanup

1. **Stop OpenClaw gateway:**
   ```bash
   openclaw gateway stop
   ```

2. **Stop gigi-p2p-example client:**
   - Press Ctrl+C in the example client terminal

3. **Uninstall the plugin (optional):**
   ```bash
   openclaw plugins uninstall gigi-p2p-bundled
   ```

## Troubleshooting

### Common Issues

1. **Peer Discovery Issues:**
   - Ensure both clients are on the same network
   - Check that mDNS and DHT are enabled
   - Try manually connecting using multiaddrs

2. **File Sharing Failures:**
   - Verify the file exists and is readable
   - Check network connectivity between peers
   - Ensure both clients are using the same Gigi P2P protocol version

3. **Group Join Failures:**
   - Verify the group name is the same in both clients
   - Check that both clients are connected to the network

4. **OpenClaw Plugin Issues:**
   - Verify the plugin is enabled in OpenClaw config
   - Check plugin logs for errors
   - Ensure the plugin bundle exists at `dist/bundle.js`

## Test Results

| Test Case | Expected Result | Actual Result | Status |
|-----------|----------------|---------------|--------|
| Basic Group Chat | Both clients can send and receive messages | | |
| File Sharing in Group | File is shared and downloaded successfully | | |
| Direct File Sharing | File is shared and downloaded successfully | | |
| Multiple Group Membership | Messages appear in the correct groups | | |
