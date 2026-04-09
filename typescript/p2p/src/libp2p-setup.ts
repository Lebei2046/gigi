import { createLibp2p } from 'libp2p';
import { webSockets } from '@libp2p/websockets';
import { webTransport } from '@libp2p/webtransport';
import { tcp } from '@libp2p/tcp';
import { noise } from '@libp2p/noise';
import { yamux } from '@libp2p/yamux';
import { GigiDnsBehaviour, defaultGigiDnsConfig } from '@gigi/mdns';
import { kadDHT } from '@libp2p/kad-dht';
import {
  circuitRelayServer,
  circuitRelayTransport,
} from '@libp2p/circuit-relay-v2';
import { identify } from '@libp2p/identify';
import { ping } from '@libp2p/ping';
import { gossipsub } from '@libp2p/gossipsub';
import { multiaddr } from '@multiformats/multiaddr';
import { generateKeyPairFromSeed } from '@libp2p/crypto/keys';
import { createLogger } from '@gigi/logging';

import { derivePeerPrivateKey, derivePeerId } from './key-derivation';

const logger = createLogger({ name: 'libp2p-setup' });

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
  mnemonic?: string;
}

export interface Libp2pInstance {
  libp2p: Awaited<ReturnType<typeof createLibp2p>>;
  gigiDns: GigiDnsBehaviour | null;
}

export async function createLibp2pInstance(
  options: CreateLibp2pOptions
): Promise<Libp2pInstance> {
  const {
    listenAddrs = ['/ip4/0.0.0.0/tcp/0'],
    bootstrapNodes = [],
    enableMdns = true,
    enableKademlia = true,
    enableRelay = true,
    mnemonic,
  } = options;

  const transports = [
    tcp(),
    webSockets(),
    webTransport(),
    circuitRelayTransport(),
  ] as any;

  // Gigi DNS for peer discovery with nickname support
  let gigiDns: GigiDnsBehaviour | null = null;

  if (enableMdns) {
    logger.info('[libp2p-setup] Enabling Gigi DNS for local peer discovery');
    // Gigi DNS will be initialized after libp2p is created
  }

  // Create services object
  const services: any = {};

  if (enableKademlia) {
    services.dht = kadDHT({ clientMode: true });
  }

  if (enableRelay) {
    services.relay = circuitRelayServer({});
  }

  services.identify = identify({
    protocolPrefix: 'gigi',
  });
  services.ping = ping();

  services.pubsub = gossipsub({
    globalSignaturePolicy: 'StrictNoSign',
  });

  // Create libp2p options without services initially
  const libp2pOptions: any = {
    addresses: { listen: listenAddrs },
    transports,
    connectionEncrypters: [noise()],
    streamMuxers: [yamux()],
  };

  // Use mnemonic for key derivation if provided
  if (mnemonic) {
    logger.info('[libp2p-setup] Using mnemonic for key derivation');
    try {
      // Derive peer ID from mnemonic
      const peerId = await derivePeerId(mnemonic);

      logger.info({
        message: '[libp2p-setup] Derived peer ID from mnemonic',
        peerId: peerId,
      });

      // Derive private key from mnemonic
      const { privateKey, publicKey } = await derivePeerPrivateKey(mnemonic);

      // Debug: check key lengths and values
      logger.debug({
        message: '[libp2p-setup] Key lengths',
        privateKeyLength: privateKey.length,
        publicKeyLength: publicKey.length,
      });
      logger.debug({
        message: '[libp2p-setup] Private key (hex)',
        privateKey: Buffer.from(privateKey).toString('hex'),
      });

      // Generate the key pair from the seed
      const privateKeyObj = await generateKeyPairFromSeed(
        'Ed25519',
        privateKey
      );

      // Set the private key in the options instead of peerId
      libp2pOptions.privateKey = privateKeyObj;
      logger.info('[libp2p-setup] Private key set in options');
      logger.info({
        message: '[libp2p-setup] Expected peer ID',
        peerId: peerId,
      });
    } catch (error) {
      logger.error({
        message: '[libp2p-setup] Error deriving peer ID from mnemonic',
        error: error,
      });
      throw error;
    }
  }

  // Add services to libp2p options
  libp2pOptions.services = services;

  const libp2p = await createLibp2p(libp2pOptions);

  // Initialize Gigi DNS after libp2p is created
  if (enableMdns) {
    const dnsConfig = {
      ...defaultGigiDnsConfig,
      nickname: options.nickname,
    };
    // Use type assertion to work around version compatibility
    gigiDns = new GigiDnsBehaviour(libp2p.peerId as any, dnsConfig);

    // Update listen addresses
    const listenAddrs = libp2p.getMultiaddrs().map((m: any) => m.toString());
    gigiDns.updateListenAddresses(listenAddrs);

    logger.info({
      message: '[libp2p-setup] Gigi DNS initialized',
      nickname: options.nickname,
    });
  }

  for (const addr of bootstrapNodes) {
    try {
      const multiAddr = multiaddr(addr);
      await libp2p.dial(multiAddr);
      logger.info({
        message: '[libp2p] Connected to bootstrap',
        address: addr,
      });
    } catch (error) {
      logger.warn({
        message: '[libp2p] Failed to connect to bootstrap',
        address: addr,
        error: error,
      });
    }
  }

  return { libp2p, gigiDns };
}
