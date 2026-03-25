export { P2pClient } from './client.js';
export type { P2pClientOptions } from './client.js';
export type { P2pConfig, PeerInfo, GroupInfo, FileInfo, ActiveDownload } from './types.js';
export { P2pEventType } from './events.js';
export type { P2pEvent } from './events.js';
export { P2pError, ErrorCode } from './errors.js';
export { FileSharingManager, CHUNK_SIZE } from './file-sharing.js';
export { GroupManager } from './group.js';
export { PeerManager } from './peer-manager.js';
export { createLibp2pInstance } from './libp2p-setup.js';
export type { SupportedProtocols, CreateLibp2pOptions } from './libp2p-setup.js';

import { P2pClient } from './client.js';
import { P2pEvent, P2pEventType } from './events.js';
import type { PeerInfo, GroupInfo, FileInfo, ActiveDownload } from './types.js';
import { P2pError, ErrorCode } from './errors.js';
import { FileSharingManager, CHUNK_SIZE } from './file-sharing.js';
import { GroupManager } from './group.js';
import { PeerManager } from './peer-manager.js';

export default {
  P2pClient,
  P2pEventType,
  P2pError,
  ErrorCode,
  FileSharingManager,
  CHUNK_SIZE,
  GroupManager,
  PeerManager,
};