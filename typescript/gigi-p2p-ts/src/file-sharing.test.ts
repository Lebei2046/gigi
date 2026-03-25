import { describe, it, expect, beforeEach, vi } from 'vitest';
import { FileSharingManager } from './file-sharing.js';

// Mock the file system
vi.mock('fs/promises', () => ({
  readFile: vi.fn().mockResolvedValue(Buffer.from('test file content')),
  writeFile: vi.fn().mockResolvedValue(undefined),
  mkdir: vi.fn().mockResolvedValue(undefined),
  stat: vi.fn().mockResolvedValue({
    size: 1024
  })
}));

// Mock crypto
vi.mock('crypto', () => ({
  randomUUID: vi.fn().mockReturnValue('mock-uuid'),
  createHash: vi.fn().mockReturnValue({
    update: vi.fn().mockReturnThis(),
    digest: vi.fn().mockReturnValue('mock-hash')
  })
}));

// Mock fs.existsSync
vi.mock('fs', () => ({
  existsSync: vi.fn().mockReturnValue(true)
}));

describe('FileSharingManager', () => {
  let fileSharingManager: FileSharingManager;

  beforeEach(() => {
    fileSharingManager = new FileSharingManager();
  });

  it('should initialize with empty shared files', () => {
    expect(fileSharingManager).toBeInstanceOf(FileSharingManager);
    expect(fileSharingManager.list()).toEqual([]);
  });

  it('should share a file and return a shared file object', async () => {
    const content = Buffer.from('test content');
    const sharedFile = await fileSharingManager.shareWithContent('test.txt', content, 'text/plain');
    expect(sharedFile).toBeDefined();
    expect(sharedFile.shareCode).toBeDefined();
    expect(typeof sharedFile.shareCode).toBe('string');
    expect(sharedFile.shareCode.length).toBeGreaterThan(0);
  });

  it('should throw an error when sharing a non-existent file', async () => {
    // Mock existsSync to return false
    const fs = await import('fs');
    fs.existsSync.mockReturnValue(false);
    
    await expect(fileSharingManager.share('./non-existent-file.txt')).rejects.toThrow('File not found: ./non-existent-file.txt');
  });

  it('should get file info by share code', async () => {
    const content = Buffer.from('test content');
    const sharedFile = await fileSharingManager.shareWithContent('test.txt', content, 'text/plain');
    const foundFile = fileSharingManager.getByShareCode(sharedFile.shareCode);
    
    expect(foundFile).toBeDefined();
    expect(foundFile?.info.name).toBe('test.txt');
    expect(foundFile?.info.size).toBe(content.length);
  });

  it('should return undefined for non-existent share code', () => {
    const foundFile = fileSharingManager.getByShareCode('non-existent-share-code');
    expect(foundFile).toBeUndefined();
  });

  it('should revoke a shared file', async () => {
    const content = Buffer.from('test content');
    const sharedFile = await fileSharingManager.shareWithContent('test.txt', content, 'text/plain');
    expect(fileSharingManager.getByShareCode(sharedFile.shareCode)?.info.revoked).toBe(false);
    
    fileSharingManager.revoke(sharedFile.shareCode);
    expect(fileSharingManager.getByShareCode(sharedFile.shareCode)?.info.revoked).toBe(true);
  });

  it('should list all shared files', async () => {
    const content1 = Buffer.from('test content 1');
    const content2 = Buffer.from('test content 2');
    await fileSharingManager.shareWithContent('test1.txt', content1, 'text/plain');
    await fileSharingManager.shareWithContent('test2.txt', content2, 'text/plain');
    
    const files = fileSharingManager.list();
    expect(files.length).toBe(2);
  });

  it('should save a file', async () => {
    const filePath = await fileSharingManager.saveFile('test.txt', [Buffer.from('test content')]);
    expect(filePath).toBeDefined();
    expect(typeof filePath).toBe('string');
  });
});
