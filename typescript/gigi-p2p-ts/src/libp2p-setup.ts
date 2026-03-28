import { createLibp2p } from 'libp2p';
import { webSockets } from '@libp2p/websockets';
import { webTransport } from '@libp2p/webtransport';
import { tcp } from '@libp2p/tcp';
import { noise } from '@libp2p/noise';
import { yamux } from '@libp2p/yamux';
import { mdns } from '@libp2p/mdns';
import { kadDHT } from '@libp2p/kad-dht';
import { circuitRelayServer, circuitRelayTransport } from '@libp2p/circuit-relay-v2';
import { identify } from '@libp2p/identify';
import { ping } from '@libp2p/ping';
import { gossipsub } from '@libp2p/gossipsub';
import { multiaddr } from '@multiformats/multiaddr';
import { createFromJSON } from '@libp2p/peer-id-factory';

export interface SupportedProtocols {
  direct: string;
  file: string;
  group: string;
}

export const PROTOCOLS: SupportedProtocols = {
  direct: '/gigi/direct/1.0.0',
  file: '/gigi/file/1.0.0',
  group: 'gigi-group',
};

export interface CreateLibp2pOptions {
  nickname: string;
  listenAddrs?: string[];
  bootstrapNodes?: string[];
  enableMdns?: boolean;
  enableKademlia?: boolean;
  enableRelay?: boolean;
  peerIdJson?: {
    id: string;
    privKey?: string;
    pubKey?: string;
  };
}

export async function createLibp2pInstance(options: CreateLibp2pOptions): Promise<ReturnType<typeof createLibp2p>> {
  const {
    listenAddrs = ['/ip4/0.0.0.0/tcp/0'],
    bootstrapNodes = [],
    enableMdns = true,
    enableKademlia = true,
    enableRelay = true,
    peerIdJson,
  } = options;

  const transports = [tcp(), webSockets(), webTransport(), circuitRelayTransport()] as any;

  const peerDiscovery: any[] = [];

  if (enableMdns) {
    console.log('[libp2p-setup] Enabling mDNS for local peer discovery');
    peerDiscovery.push(mdns({
      interval: 10000 // 10 second interval for mDNS queries
    }));
  }

  const services: any = {};

  if (enableKademlia) {
    services.dht = kadDHT({ clientMode: true });
  }

  if (enableRelay) {
    services.relay = circuitRelayServer({});
  }

  services.identify = identify({
    protocolPrefix: 'gigi'
  });
  services.ping = ping();

  services.pubsub = gossipsub({
    globalSignaturePolicy: 'StrictNoSign',
  });

  const libp2pOptions: any = {
    addresses: { listen: listenAddrs },
    transports,
    peerDiscovery,
    connectionEncrypters: [noise()],
    streamMuxers: [yamux()],
    services,
  };

  // Use pre-generated peer ID if provided
  if (peerIdJson) {
    console.log('[libp2p-setup] Using pre-generated peer ID');
    libp2pOptions.peerId = await createFromJSON(peerIdJson);
  }

  const libp2p = await createLibp2p(libp2pOptions);

  for (const addr of bootstrapNodes) {
    try {
      const multiAddr = multiaddr(addr);
      await libp2p.dial(multiAddr);
      console.log(`[libp2p] Connected to bootstrap: ${addr}`);
    } catch (error) {
      console.warn(`[libp2p] Failed to connect to bootstrap ${addr}:`, error);
    }
  }

  return libp2p;
}