export { P2pClient } from './client';
export type { P2pClientOptions } from './client';
export type {
  P2pConfig,
  PeerInfo,
  GroupInfo,
  FileInfo,
  ActiveDownload,
} from './types';
export { P2pEventType, eventEmitter } from './events';
export type { P2pEvent } from './events';
export { P2pError, ErrorCode } from './errors';
export { FileSharingManager, CHUNK_SIZE } from './file-sharing';
export { GroupManager } from './group';
export { PeerManager } from './peer-manager';
export { createLibp2pInstance } from './libp2p-setup';
export type { SupportedProtocols, CreateLibp2pOptions } from './libp2p-setup';
export {
  derivePeerId,
  deriveGroupId,
  derivePeerPrivateKey,
  generateMnemonic,
} from './key-derivation';

import { P2pClient } from './client';
import { P2pEventType } from './events';
import { P2pError, ErrorCode } from './errors';
import { FileSharingManager, CHUNK_SIZE } from './file-sharing';
import { GroupManager } from './group';
import { PeerManager } from './peer-manager';
import {
  derivePeerId,
  deriveGroupId,
  derivePeerPrivateKey,
  generateMnemonic,
} from './key-derivation';

export default {
  P2pClient,
  P2pEventType,
  P2pError,
  ErrorCode,
  FileSharingManager,
  CHUNK_SIZE,
  GroupManager,
  PeerManager,
  derivePeerId,
  deriveGroupId,
  derivePeerPrivateKey,
  generateMnemonic,
};
