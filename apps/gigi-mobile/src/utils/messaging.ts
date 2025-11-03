import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export async function greet(name: string): Promise<string> {
  return await invoke('greet', { name });
}

/**
 * 获取PeerId
 * @param privKey 私钥(HEX字符串)
 * @returns PeerId
 */
export async function getPeerId(privKey: Uint8Array): Promise<string> {
  const hex = toHexString(privKey);
  return await invoke('get_peer_id', { privKey: hex });
}

/**
 * 订阅指定主题
 * @param topic 主题名称
 */
export async function subscribeTopic(topic: string): Promise<void> {
  await invoke('subscribe_topic', { topic });
}

/**
 * 取消订阅指定主题
 * @param topic 主题名称
 */
export async function unsubscribeTopic(topic: string): Promise<void> {
  await invoke('unsubscribe_topic', { topic });
}

/**
 * 发送消息到指定主题
 * @param topic 主题名称
 * @param message 消息内容
 */
export async function sendMessage(topic: string, message: string): Promise<void> {
  await invoke('send_message', { topic, message });
}

/**
 * 获取当前发现的节点列表
 * @returns 节点列表
 */
export async function getPeers(): Promise<Array<[string, string[]]>> {
  return await invoke('get_peers');
}

export interface MessageReceivedEvent {
  topic: string;
  data: string;
  sender: string;
}
export type MessageReceivedCallback = (event: MessageReceivedEvent) => void;

/**
 * 监听消息接收事件
 * @param callback 回调函数，参数为 { topic: string, data: string, sender: string }
 * @returns 取消监听的函数
 */
export async function onMessageReceived(
  callback: MessageReceivedCallback,
): Promise<UnlistenFn> {
  return await listen('message-received', (event) => {
    callback(event.payload as MessageReceivedEvent);
  });
}

export interface PeerDiscoveredEvent {
  id: string;
  addresses: string[];
}
export type PeerDiscoveredCallback = (event: PeerDiscoveredEvent) => void;

/**
 * 监听节点发现事件
 * @param callback 回调函数，参数为 { id: string, addresses: string[] }
 * @returns 取消监听的函数
 */
export async function onPeerDiscovered(
  callback: PeerDiscoveredCallback,
): Promise<UnlistenFn> {
  return await listen('peer-discovered', (event) => {
    callback(event.payload as PeerDiscoveredEvent);
  });
}

// Convert bytes to hex string
const toHexString = (bytes: Uint8Array) => {
  return Array.from(bytes, (byte) => {
    return ('0' + (byte & 0xff).toString(16)).slice(-2);
  }).join('');
};
