import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { P2pClient } from '../../typescript/p2p/dist/client.js';
import { AmpMessageFactory } from '../../typescript/amp/dist/index.js';
import { createLogger } from '../../typescript/logging/dist/index.js';

const logger = createLogger({ name: 'gigi-openclaw-test' });

describe('Gigi OpenClaw P2P Agent Test', () => {
  let p2pClient: P2pClient;
  let testPeerId: string;

  // 目标 OpenClaw 节点的对等节点 ID（从提供的助记词派生）
  const targetPeerId = '12D3KooWKrVwR4tFJgMBt1LoEFN4eVVyJsP9cNGZpfYkwVGK8Cac';

  beforeEach(async () => {
    // 实例化 P2pClient，不使用助记词
    p2pClient = new P2pClient({
      nickname: 'Test Client',
      config: {
        bootstrapNodes: [],
        enableKademlia: true,
        enableRelay: true,
        enableMdns: true,
        listenAddrs: ['/ip4/0.0.0.0/tcp/0', '/ip4/0.0.0.0/tcp/0/ws'],
      },
      // 不使用助记词，让 P2pClient 生成新的对等节点 ID
    });

    // 启动 P2pClient
    await p2pClient.start();
    testPeerId = p2pClient.getPeerId();
    logger.info(`Test peer ID: ${testPeerId}`);
  });

  afterEach(async () => {
    // 停止 P2pClient
    if (p2pClient && p2pClient.isStarted()) {
      await p2pClient.stop();
    }
  });

  it('should discover the OpenClaw node, join gigi-agents group, and send group message to main agent', async () => {
    logger.info('Starting test to discover OpenClaw node, join group, and send message...');
    logger.info(`Target OpenClaw peer ID: ${targetPeerId}`);

    // 监听所有 peer-discovered 事件，记录发现的所有节点
    p2pClient.onEvent(async (event) => {
      if (event.type === 'peer-discovered') {
        logger.info(`Discovered peer: ${event.peerId} (${event.nickname})`);
      }
    });

    // 创建 AMP 消息，发送给 main agent
    const ampMessage = AmpMessageFactory.createTextMessage(
      'Hello from Gigi OpenClaw test!',
      { type: 'specific', agentIds: ['main'] },
      {
        id: testPeerId,
        name: 'Test Client',
        type: 'agent'
      }
    );

    // 先等待发现目标节点
    let peerDiscovered = false;
    
    // 检查是否已经发现了目标节点
    const peers = p2pClient.listPeers();
    for (const peer of peers) {
      if (peer.peerId === targetPeerId) {
        peerDiscovered = true;
        logger.info(`Target OpenClaw node already discovered: ${peer.peerId} (${peer.nickname})`);
        break;
      }
    }
    
    // 如果还没有发现，等待发现
    if (!peerDiscovered) {
      logger.info('Waiting for peer discovery...');
      
      // 监听 peer-discovered 事件
      const peerDiscoveredPromise = new Promise<void>((resolve) => {
        const unsubscribe = p2pClient.onEvent(async (event) => {
          if (event.type === 'peer-discovered' && event.peerId === targetPeerId) {
            logger.info(`Discovered target OpenClaw node: ${event.peerId} (${event.nickname})`);
            unsubscribe();
            resolve();
          }
        });
      });

      // 等待发现目标节点，超时 60 秒
      await Promise.race([
        peerDiscoveredPromise,
        new Promise<void>((_, reject) => {
          setTimeout(() => reject(new Error('Peer discovery timeout')), 60000);
        })
      ]);
    }
    
    // 等待连接建立
    await new Promise(resolve => setTimeout(resolve, 5000));

    // 加入 gigi-agents 组
    logger.info('Joining gigi-agents group...');
    await p2pClient.joinGroup('gigi-agents');
    logger.info('Joined gigi-agents group successfully');

    // 等待组加入完成
    await new Promise(resolve => setTimeout(resolve, 5000));

    // 等待来自 main agent 的响应 - 设置监听器在发送消息之前
    const responsePromise = new Promise<string>((resolve) => {
      // 监听所有事件，确保我们不会错过任何消息
      const unsubscribe = p2pClient.onEvent(async (event) => {
        logger.info(`Received event: ${event.type}`);
        if (event.type === 'direct-message') {
          logger.info(`Received direct message from: ${event.from}`);
          try {
            const messageData = typeof event.message === 'string' ? JSON.parse(event.message) : event.message;
            logger.info(`Message data: ${JSON.stringify(messageData)}`);
            if (messageData.type === 'text' && messageData.sender && messageData.sender.id === 'main') {
              logger.info(`Received response from main agent: ${messageData.content}`);
              unsubscribe();
              resolve(messageData.content);
            }
          } catch (error) {
            logger.error('Error parsing response message:', error);
          }
        }
      });
    });

    // 发送组消息到 gigi-agents 组
    logger.info('Sending group message to gigi-agents group...');
    try {
      // Format the message correctly for the P2pClient
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await p2pClient.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent successfully');
    } catch (error) {
      logger.error('Error sending group message:', error);
      // 再次尝试发送消息
      const messageContent = {
        type: 'text' as const,
        text: JSON.stringify(ampMessage)
      };
      await p2pClient.sendGroupMessage('gigi-agents', messageContent);
      logger.info('Group message sent after retry');
    }

    // 等待响应，超时 120 秒
    const response = await Promise.race([
      responsePromise,
      new Promise<string>((_, reject) => {
        setTimeout(() => reject(new Error('Response timeout')), 120000);
      })
    ]);

    logger.info(`Response received: ${response}`);
    expect(response).toBeDefined();
    expect(typeof response).toBe('string');
  }, 180000); // 增加测试超时时间到 180 秒
});
