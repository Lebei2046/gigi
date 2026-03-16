import { createHash } from 'crypto';
import { readFile, writeFile, mkdir } from 'fs/promises';
import { existsSync } from 'fs';
import { join, basename } from 'path';
import type { FileInfo, SharedFile } from './types.js';

export const CHUNK_SIZE = 256 * 1024;

export class FileSharingManager {
  private files: Map<string, SharedFile> = new Map();
  private shareCodeIndex: Map<string, string> = new Map();
  private outputDirectory: string;

  constructor(outputDirectory: string = './downloads') {
    this.outputDirectory = outputDirectory;
  }

  async share(filePath: string): Promise<SharedFile> {
    if (!existsSync(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }

    const content = await readFile(filePath);
    const hash = this.calculateHash(content);
    const fileId = crypto.randomUUID();
    const shareCode = this.generateShareCode(basename(filePath));
    const chunkCount = Math.ceil(content.length / CHUNK_SIZE);
    const mimeType = this.guessMimeType(filePath);

    const info: FileInfo = {
      fileId,
      shareCode,
      name: basename(filePath),
      size: content.length,
      mimeType,
      chunkCount,
      hash,
      createdAt: Date.now(),
      revoked: false,
    };

    const sharedFile: SharedFile = { fileId, shareCode, info };

    this.files.set(fileId, sharedFile);
    this.shareCodeIndex.set(shareCode, fileId);

    return sharedFile;
  }

  async shareWithContent(name: string, content: Uint8Array, mimeType: string = 'application/octet-stream'): Promise<SharedFile> {
    const hash = this.calculateHash(content);
    const fileId = crypto.randomUUID();
    const shareCode = this.generateShareCode(name);
    const chunkCount = Math.ceil(content.length / CHUNK_SIZE);

    const info: FileInfo = {
      fileId,
      shareCode,
      name,
      size: content.length,
      mimeType,
      chunkCount,
      hash,
      createdAt: Date.now(),
      revoked: false,
    };

    const sharedFile: SharedFile = { fileId, shareCode, info };

    this.files.set(fileId, sharedFile);
    this.shareCodeIndex.set(shareCode, fileId);

    return sharedFile;
  }

  revoke(shareCode: string): void {
    const fileId = this.shareCodeIndex.get(shareCode);
    if (!fileId) return;

    const file = this.files.get(fileId);
    if (file) {
      file.info.revoked = true;
    }
  }

  get(fileId: string): SharedFile | undefined {
    return this.files.get(fileId);
  }

  getByShareCode(shareCode: string): SharedFile | undefined {
    const fileId = this.shareCodeIndex.get(shareCode);
    return fileId ? this.files.get(fileId) : undefined;
  }

  list(): SharedFile[] {
    return Array.from(this.files.values()).filter(f => !f.info.revoked);
  }

  listAll(): SharedFile[] {
    return Array.from(this.files.values());
  }

  async saveFile(filename: string, chunks: Uint8Array[]): Promise<string> {
    if (!existsSync(this.outputDirectory)) {
      await mkdir(this.outputDirectory, { recursive: true });
    }

    const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
    const data = new Uint8Array(totalLength);
    let offset = 0;
    for (const chunk of chunks) {
      data.set(chunk, offset);
      offset += chunk.length;
    }

    const filePath = join(this.outputDirectory, filename);
    await writeFile(filePath, Buffer.from(data));

    return filePath;
  }

  getChunk(fileId: string, chunkIndex: number, _filePath: string): Uint8Array {
    return new Uint8Array(0);
  }

  private calculateHash(data: Uint8Array): string {
    const hash = createHash('sha256');
    hash.update(Buffer.from(data));
    return hash.digest('hex');
  }

  private generateShareCode(filename: string): string {
    const timestamp = Date.now().toString(36);
    const input = `${filename}:${timestamp}`;
    const hash = createHash('blake3').update(input).digest('hex');
    return hash.substring(0, 8);
  }

  private guessMimeType(filePath: string): string {
    const ext = filePath.split('.').pop()?.toLowerCase() || '';

    const mimeTypes: Record<string, string> = {
      'jpg': 'image/jpeg',
      'jpeg': 'image/jpeg',
      'png': 'image/png',
      'gif': 'image/gif',
      'webp': 'image/webp',
      'svg': 'image/svg+xml',
      'mp4': 'video/mp4',
      'webm': 'video/webm',
      'mp3': 'audio/mpeg',
      'wav': 'audio/wav',
      'pdf': 'application/pdf',
      'txt': 'text/plain',
      'html': 'text/html',
      'css': 'text/css',
      'js': 'application/javascript',
      'json': 'application/json',
      'xml': 'application/xml',
      'zip': 'application/zip',
    };

    return mimeTypes[ext] || 'application/octet-stream';
  }
}