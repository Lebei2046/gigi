import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { P2pClient } from '../client';
import type { MessageContentInput } from '../types';

// 不使用模拟，使用真实的实现
describe('P2pClient (Real Implementation)', () => {
  let client: P2pClient;
  const config = {
    nickname: 'test-client',
    config: {
      bootstrapNodes: [],
      enableKademlia: false,
      enableRelay: false,
      enableMdns: false,
      listenAddrs: ['/ip4/127.0.0.1/tcp/15000'],
    },
  };

  beforeEach(async () => {
    client = new P2pClient(config);
  });

  afterEach(async () => {
    if (client) {
      await client.stop();
    }
  });

  it('should initialize with the correct configuration', () => {
    expect(client).toBeInstanceOf(P2pClient);
  });

  it('should start and stop successfully', async () => {
    await client.start();
    expect(client.isStarted()).toBe(true);

    await client.stop();
    expect(client.isStarted()).toBe(false);
  });

  it('should return the correct peer ID', async () => {
    await client.start();
    const peerId = client.getPeerId();
    expect(typeof peerId).toBe('string');
    expect(peerId.length).toBeGreaterThan(0);
  });

  it('should return the correct multiaddrs', async () => {
    await client.start();
    const multiaddrs = client.getMultiaddrs();
    expect(Array.isArray(multiaddrs)).toBe(true);
    expect(multiaddrs.length).toBeGreaterThan(0);
    multiaddrs.forEach((addr) => {
      expect(typeof addr).toBe('string');
      expect(addr.startsWith('/ip4/')).toBe(true);
    });
  });

  it('should handle starting an already started client', async () => {
    await client.start();
    expect(client.isStarted()).toBe(true);

    // Should throw an error when starting again
    await expect(client.start()).rejects.toThrow('P2P client already started');
  });

  it('should handle stopping an already stopped client', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should not throw an error when stopping
    await expect(client.stop()).resolves.not.toThrow();
  });

  it('should handle sending message when client is not started', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should throw an error when sending message
    await expect(
      client.sendDirectMessage(
        '12D3KooWJzRg78CLWfUjH1W8n6X8C4YkZ8q7Q3X9P8L7M6N5B4A3',
        'Hello'
      )
    ).rejects.toThrow('P2P client not started');
  });

  it('should handle sharing file when client is not started', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should throw an error when sharing file
    await expect(client.shareFile('./test.txt')).rejects.toThrow(
      'P2P client not started'
    );
  });

  it('should handle downloading file when client is not started', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should throw an error when downloading file
    await expect(
      client.downloadFile('test-peer', 'mock-share-code')
    ).rejects.toThrow('P2P client not started');
  });

  it('should handle joining group when client is not started', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should throw an error when joining group
    await expect(client.joinGroup('general')).rejects.toThrow(
      'P2P client not started'
    );
  });

  it('should handle sending group message when client is not started', async () => {
    // Client is not started yet
    expect(client.isStarted()).toBe(false);

    // Should throw an error when sending group message
    await expect(
      client.sendGroupMessage('general', {
        type: 'text',
        text: 'Hello everyone',
      } as MessageContentInput)
    ).rejects.toThrow('P2P client not started');
  });

  it('should register and remove event listeners', async () => {
    await client.start();

    const eventListener = vi.fn();
    const removeListener = client.onEvent(eventListener);
    expect(typeof removeListener).toBe('function');

    removeListener();
    // Just verify the removeListener function exists and is callable
    expect(typeof removeListener).toBe('function');
  });
});
