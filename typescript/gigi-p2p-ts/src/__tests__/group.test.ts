import { describe, it, expect, beforeEach } from 'vitest';
import { GroupManager } from '../group.js';

describe('GroupManager', () => {
  let groupManager: GroupManager;

  beforeEach(() => {
    groupManager = new GroupManager();
  });

  it('should initialize with empty groups', () => {
    expect(groupManager).toBeInstanceOf(GroupManager);
    expect(groupManager.list()).toEqual([]);
  });

  it('should join a group', () => {
    const groupName = 'general';
    const topic = '/gigi/group/general';
    groupManager.join(groupName, topic);

    const groups = groupManager.list();
    expect(groups.length).toBe(1);
    expect(groups[0].name).toBe(groupName);
    expect(groups[0].topic).toBe(topic);
  });

  it('should leave a group', () => {
    const groupName = 'general';
    const topic = '/gigi/group/general';
    groupManager.join(groupName, topic);
    expect(groupManager.list().length).toBe(1);

    groupManager.leave(groupName);
    expect(groupManager.list()).toEqual([]);
  });

  it('should check if a group exists', () => {
    const groupName = 'general';
    const topic = '/gigi/group/general';
    groupManager.join(groupName, topic);

    expect(groupManager.has(groupName)).toBe(true);
    expect(groupManager.has('non-existent-group')).toBe(false);
  });

  it('should get a group by name', () => {
    const groupName = 'general';
    const topic = '/gigi/group/general';
    groupManager.join(groupName, topic);

    const group = groupManager.get(groupName);
    expect(group).toBeDefined();
    expect(group?.name).toBe(groupName);
    expect(group?.topic).toBe(topic);
  });

  it('should list all groups', () => {
    groupManager.join('general', '/gigi/group/general');
    groupManager.join('random', '/gigi/group/random');

    const groups = groupManager.list();
    expect(groups.length).toBe(2);
    expect(groups.map((g) => g.name)).toEqual(['general', 'random']);
  });

  it('should get all group names', () => {
    groupManager.join('general', '/gigi/group/general');
    groupManager.join('random', '/gigi/group/random');

    const groupNames = groupManager.names();
    expect(groupNames.length).toBe(2);
    expect(groupNames).toEqual(['general', 'random']);
  });

  it('should not join the same group multiple times', () => {
    const groupName = 'general';
    const topic = '/gigi/group/general';
    groupManager.join(groupName, topic);
    groupManager.join(groupName, topic);

    const groups = groupManager.list();
    expect(groups.length).toBe(1);
  });

  it('should not throw when leaving a non-existent group', () => {
    expect(() => groupManager.leave('non-existent-group')).not.toThrow();
  });
});
