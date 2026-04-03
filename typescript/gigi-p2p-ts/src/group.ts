import type { GroupInfo } from './types.js';

export class GroupManager {
  private groups: Map<string, GroupInfo> = new Map();

  join(name: string, topic: string): void {
    const info: GroupInfo = {
      name,
      topic,
      joinedAt: Date.now(),
    };

    this.groups.set(name, info);
  }

  leave(name: string): void {
    this.groups.delete(name);
  }

  has(name: string): boolean {
    return this.groups.has(name);
  }

  get(name: string): GroupInfo | undefined {
    return this.groups.get(name);
  }

  list(): GroupInfo[] {
    return Array.from(this.groups.values());
  }

  names(): string[] {
    return Array.from(this.groups.keys());
  }
}
