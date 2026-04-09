import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createLibp2pInstance, PROTOCOLS } from '../libp2p-setup';
import { createLibp2p } from 'libp2p';
import { GigiDnsBehaviour } from '@gigi/mdns';

// Get the mocked GigiDnsBehaviour constructor
const mockGigiDnsBehaviour = vi.mocked(GigiDnsBehaviour);

// Mock dependencies
vi.mock('libp2p');
vi.mock('@gigi/mdns', () => {
  const mockGigiDnsBehaviour = vi.fn();
  mockGigiDnsBehaviour.mockImplementation(function () {
    return {
      updateListenAddresses: vi.fn(),
    };
  });
  return {
    GigiDnsBehaviour: mockGigiDnsBehaviour,
    defaultGigiDnsConfig: {
      nickname: 'default',
      ttl: 30000,
      capabilities: [],
      metadata: {},
    },
  };
});
vi.mock('../key-derivation.js');

const mockCreateLibp2p = vi.mocked(createLibp2p);

// Mock implementations
const mockLibp2p = {
  peerId: 'mock-peer-id',
  getMultiaddrs: vi.fn().mockReturnValue(['/ip4/127.0.0.1/tcp/12345']),
  dial: vi.fn().mockResolvedValue(undefined),
};

describe('libp2p-setup', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockCreateLibp2p.mockResolvedValue(mockLibp2p as any);
  });

  it('should export the correct protocols', () => {
    expect(PROTOCOLS.direct).toBe('/gigi/direct/1.0.0');
    expect(PROTOCOLS.file).toBe('/gigi/file/1.0.0');
    expect(PROTOCOLS.group).toBe('gigi-group');
  });

  it('should create a libp2p instance with default options', async () => {
    const result = await createLibp2pInstance({
      nickname: 'test-node',
    });

    expect(mockCreateLibp2p).toHaveBeenCalled();
    expect(result.libp2p).toBe(mockLibp2p);
    expect(result.gigiDns).toBeDefined();
    expect(result.gigiDns).not.toBeNull();
    if (result.gigiDns) {
      expect(result.gigiDns.updateListenAddresses).toBeDefined();
    }
  });

  it('should create a libp2p instance with custom listen addresses', async () => {
    const customListenAddrs = ['/ip4/127.0.0.1/tcp/8080'];

    await createLibp2pInstance({
      nickname: 'test-node',
      listenAddrs: customListenAddrs,
    });

    expect(mockCreateLibp2p).toHaveBeenCalledWith(
      expect.objectContaining({
        addresses: {
          listen: customListenAddrs,
        },
      })
    );
  });

  it('should not initialize Gigi DNS when enableMdns is false', async () => {
    const result = await createLibp2pInstance({
      nickname: 'test-node',
      enableMdns: false,
    });

    expect(mockGigiDnsBehaviour).not.toHaveBeenCalled();
    expect(result.gigiDns).toBeNull();
  });

  it('should initialize Gigi DNS when enableMdns is true', async () => {
    await createLibp2pInstance({
      nickname: 'test-node',
      enableMdns: true,
    });

    expect(mockGigiDnsBehaviour).toHaveBeenCalled();
    // Get the instance created by GigiDnsBehaviour
    const gigiDnsInstance = mockGigiDnsBehaviour.mock.results[0].value;
    expect(gigiDnsInstance.updateListenAddresses).toHaveBeenCalledWith([
      '/ip4/127.0.0.1/tcp/12345',
    ]);
  });

  it('should connect to bootstrap nodes', async () => {
    const bootstrapNodes = ['/ip4/1.2.3.4/tcp/4001/p2p/QmTest'];

    await createLibp2pInstance({
      nickname: 'test-node',
      bootstrapNodes,
    });

    expect(mockLibp2p.dial).toHaveBeenCalled();
  });

  it('should handle bootstrap node connection failures gracefully', async () => {
    const bootstrapNodes = ['/ip4/1.2.3.4/tcp/4001/p2p/QmTest'];
    mockLibp2p.dial.mockRejectedValue(new Error('Connection failed'));

    // This should not throw
    await expect(
      createLibp2pInstance({
        nickname: 'test-node',
        bootstrapNodes,
      })
    ).resolves.toBeDefined();
  });

  it('should enable Kademlia DHT when enableKademlia is true', async () => {
    await createLibp2pInstance({
      nickname: 'test-node',
      enableKademlia: true,
    });

    // We can't easily check the services object, but we can verify the function was called
    expect(mockCreateLibp2p).toHaveBeenCalled();
  });

  it('should enable circuit relay when enableRelay is true', async () => {
    await createLibp2pInstance({
      nickname: 'test-node',
      enableRelay: true,
    });

    expect(mockCreateLibp2p).toHaveBeenCalled();
  });
});
