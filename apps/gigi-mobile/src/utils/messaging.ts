import { invoke } from '@tauri-apps/api/core';

/**
 * 获取PeerId
 * @param privKey 私钥(HEX字符串)
 * @returns PeerId
 */
export async function tryGetPeerId(privKey: Uint8Array): Promise<string> {
  const hex = toHexString(privKey);
  return await invoke('try_get_peer_id', { privKey: hex });
}

// Convert bytes to hex string
const toHexString = (bytes: Uint8Array) => {
  return Array.from(bytes, (byte) => {
    return ('0' + (byte & 0xff).toString(16)).slice(-2);
  }).join('');
};
