// Tests for message types

import { describe, it, expect } from 'vitest';
import {
  isTextMessage,
  isFileMessage,
  isGroupShareMessage,
  isFileShareMessage,
  TextMessage,
  FileMessage,
  GroupShareMessage,
  FileShareMessage,
  GigiMessage,
  MessageContent,
  MessageContentInput,
  SenderInfo,
  TargetInfo,
} from '../index';

describe('Message type guards', () => {
  const mockSender: SenderInfo = {
    id: 'sender1',
    name: 'Sender',
    type: 'owner',
  };

  const mockTarget: TargetInfo = {
    type: 'all',
  };

  it('should identify text message', () => {
    const textMessage: TextMessage = {
      type: 'text',
      content: 'Hello',
      target: mockTarget,
      sender: mockSender,
      timestamp: Date.now(),
      id: 'msg1',
    };

    expect(isTextMessage(textMessage)).toBe(true);
    expect(isFileMessage(textMessage)).toBe(false);
    expect(isGroupShareMessage(textMessage)).toBe(false);
    expect(isFileShareMessage(textMessage)).toBe(false);
  });

  it('should identify file message', () => {
    const fileMessage: FileMessage = {
      type: 'file',
      filename: 'test.txt',
      fileSize: 1024,
      fileHash: 'hash123',
      target: mockTarget,
      sender: mockSender,
      timestamp: Date.now(),
      id: 'msg2',
    };

    expect(isTextMessage(fileMessage)).toBe(false);
    expect(isFileMessage(fileMessage)).toBe(true);
    expect(isGroupShareMessage(fileMessage)).toBe(false);
    expect(isFileShareMessage(fileMessage)).toBe(false);
  });

  it('should identify group share message', () => {
    const groupShareMessage: GroupShareMessage = {
      type: 'shareGroup',
      groupId: 'group1',
      groupName: 'Test Group',
      inviterNickname: 'Inviter',
      target: mockTarget,
      sender: mockSender,
      timestamp: Date.now(),
      id: 'msg3',
    };

    expect(isTextMessage(groupShareMessage)).toBe(false);
    expect(isFileMessage(groupShareMessage)).toBe(false);
    expect(isGroupShareMessage(groupShareMessage)).toBe(true);
    expect(isFileShareMessage(groupShareMessage)).toBe(false);
  });

  it('should identify file share message', () => {
    const fileShareMessage: FileShareMessage = {
      type: 'fileShare',
      shareCode: 'share123',
      filename: 'test.txt',
      fileSize: 1024,
      fileType: 'text/plain',
      target: mockTarget,
      sender: mockSender,
      timestamp: Date.now(),
      id: 'msg4',
    };

    expect(isTextMessage(fileShareMessage)).toBe(false);
    expect(isFileMessage(fileShareMessage)).toBe(false);
    expect(isGroupShareMessage(fileShareMessage)).toBe(false);
    expect(isFileShareMessage(fileShareMessage)).toBe(true);
  });

  it('should return false for non-message objects', () => {
    const nonMessage = { type: 'unknown', content: 'Hello' };
    expect(isTextMessage(nonMessage)).toBe(false);
    expect(isFileMessage(nonMessage)).toBe(false);
    expect(isGroupShareMessage(nonMessage)).toBe(false);
    expect(isFileShareMessage(nonMessage)).toBe(false);
  });

  it('should return false for null/undefined', () => {
    expect(isTextMessage(null)).toBe(false);
    expect(isFileMessage(undefined)).toBe(false);
    expect(isGroupShareMessage(null)).toBe(false);
    expect(isFileShareMessage(undefined)).toBe(false);
  });
});

describe('Message type interfaces', () => {
  it('should accept valid text message', () => {
    const textMessage: TextMessage = {
      type: 'text',
      content: 'Hello',
      target: { type: 'all' },
      sender: { id: 'sender1', name: 'Sender', type: 'owner' },
      timestamp: Date.now(),
      id: 'msg1',
    };

    expect(textMessage.type).toBe('text');
    expect(typeof textMessage.content).toBe('string');
  });

  it('should accept valid file message', () => {
    const fileMessage: FileMessage = {
      type: 'file',
      filename: 'test.txt',
      fileSize: 1024,
      fileHash: 'hash123',
      target: { type: 'all' },
      sender: { id: 'sender1', name: 'Sender', type: 'owner' },
      timestamp: Date.now(),
      id: 'msg2',
    };

    expect(fileMessage.type).toBe('file');
    expect(typeof fileMessage.filename).toBe('string');
    expect(typeof fileMessage.fileSize).toBe('number');
  });

  it('should accept valid group share message', () => {
    const groupShareMessage: GroupShareMessage = {
      type: 'shareGroup',
      groupId: 'group1',
      groupName: 'Test Group',
      inviterNickname: 'Inviter',
      target: { type: 'all' },
      sender: { id: 'sender1', name: 'Sender', type: 'owner' },
      timestamp: Date.now(),
      id: 'msg3',
    };

    expect(groupShareMessage.type).toBe('shareGroup');
    expect(typeof groupShareMessage.groupId).toBe('string');
  });

  it('should accept valid file share message', () => {
    const fileShareMessage: FileShareMessage = {
      type: 'fileShare',
      shareCode: 'share123',
      filename: 'test.txt',
      fileSize: 1024,
      fileType: 'text/plain',
      target: { type: 'all' },
      sender: { id: 'sender1', name: 'Sender', type: 'owner' },
      timestamp: Date.now(),
      id: 'msg4',
    };

    expect(fileShareMessage.type).toBe('fileShare');
    expect(typeof fileShareMessage.shareCode).toBe('string');
  });
});

describe('MessageContent types', () => {
  it('should accept valid text content', () => {
    const textContent: MessageContent = {
      type: 'text',
      text: 'Hello',
      fromPeerId: 'peer1',
      fromNickname: 'Peer',
    };

    expect(textContent.type).toBe('text');
    expect(typeof textContent.text).toBe('string');
  });

  it('should accept valid file share content', () => {
    const fileShareContent: MessageContent = {
      type: 'fileShare',
      shareCode: 'share123',
      filename: 'test.txt',
      fileSize: 1024,
      fileType: 'text/plain',
      fromPeerId: 'peer1',
      fromNickname: 'Peer',
    };

    expect(fileShareContent.type).toBe('fileShare');
    expect(typeof fileShareContent.shareCode).toBe('string');
  });

  it('should accept valid share group content', () => {
    const shareGroupContent: MessageContent = {
      type: 'shareGroup',
      groupId: 'group1',
      groupName: 'Test Group',
      inviterNickname: 'Inviter',
      fromPeerId: 'peer1',
      fromNickname: 'Peer',
    };

    expect(shareGroupContent.type).toBe('shareGroup');
    expect(typeof shareGroupContent.groupId).toBe('string');
  });
});

describe('MessageContentInput types', () => {
  it('should accept valid text input', () => {
    const textInput: MessageContentInput = {
      type: 'text',
      text: 'Hello',
    };

    expect(textInput.type).toBe('text');
    expect(typeof textInput.text).toBe('string');
  });

  it('should accept valid file share input', () => {
    const fileShareInput: MessageContentInput = {
      type: 'fileShare',
      shareCode: 'share123',
      filename: 'test.txt',
      fileSize: 1024,
      fileType: 'text/plain',
    };

    expect(fileShareInput.type).toBe('fileShare');
    expect(typeof fileShareInput.shareCode).toBe('string');
  });

  it('should accept valid share group input', () => {
    const shareGroupInput: MessageContentInput = {
      type: 'shareGroup',
      groupId: 'group1',
      groupName: 'Test Group',
      inviterNickname: 'Inviter',
    };

    expect(shareGroupInput.type).toBe('shareGroup');
    expect(typeof shareGroupInput.groupId).toBe('string');
  });
});
