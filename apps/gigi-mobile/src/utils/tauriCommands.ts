import { invoke } from '@tauri-apps/api/core'

// Auth Commands

/**
 * Check if an account exists
 */
export async function authCheckAccount(): Promise<boolean> {
  return await invoke<boolean>('plugin:gigi-p2p|auth_check_account')
}

/**
 * Signup with mnemonic and password
 * @param mnemonic - The mnemonic phrase
 * @param password - The password for encryption
 * @param name - The user's name
 * @param groupName - Optional group name to create
 */
export async function authSignup(
  mnemonic: string,
  password: string,
  name: string,
  groupName?: string
): Promise<AccountInfo> {
  return await invoke<AccountInfo>('plugin:gigi-p2p|auth_signup', {
    mnemonic,
    password,
    name,
    groupName,
  })
}

/**
 * Login with password and initialize P2P client (combined command)
 * @param password - The password to decrypt the mnemonic
 */
export async function authLoginWithP2P(password: string): Promise<AccountInfo> {
  return await invoke<AccountInfo>('plugin:gigi-p2p|auth_login_with_p2p', {
    password,
  })
}

/**
 * Get account info (without exposing sensitive data)
 */
export async function authGetAccountInfo(): Promise<AccountInfo | null> {
  return await invoke<AccountInfo | null>(
    'plugin:gigi-p2p|auth_get_account_info'
  )
}

/**
 * Delete account and all related data
 */
export async function authDeleteAccount(): Promise<void> {
  return await invoke<void>('plugin:gigi-p2p|auth_delete_account')
}

/**
 * Verify password without exposing account data
 * @param password - The password to verify
 */
export async function authVerifyPassword(password: string): Promise<boolean> {
  return await invoke<boolean>('plugin:gigi-p2p|auth_verify_password', {
    password,
  })
}

// Group Commands

/**
 * Create a new group
 * @param groupId - The group ID
 * @param groupName - The group name
 * @param joined - Whether the user joined (true) or created (false) the group
 */
export async function groupCreate(
  groupId: string,
  groupName: string,
  joined: boolean
): Promise<GroupInfo> {
  return await invoke<GroupInfo>('plugin:gigi-p2p|group_create', {
    groupId,
    groupName,
    joined,
  })
}

/**
 * Join an existing group
 * @param groupId - The group ID
 * @param groupName - The group name
 */
export async function groupJoin(
  groupId: string,
  groupName: string
): Promise<GroupInfo> {
  return await invoke<GroupInfo>('plugin:gigi-p2p|group_join', {
    groupId,
    groupName,
  })
}

/**
 * Get all groups
 */
export async function groupGetAll(): Promise<GroupInfo[]> {
  return await invoke<GroupInfo[]>('plugin:gigi-p2p|group_get_all')
}

/**
 * Get a specific group by ID
 * @param groupId - The group ID
 */
export async function groupGet(groupId: string): Promise<GroupInfo | null> {
  return await invoke<GroupInfo | null>('plugin:gigi-p2p|group_get', {
    groupId,
  })
}

/**
 * Delete a group
 * @param groupId - The group ID
 */
export async function groupDelete(groupId: string): Promise<void> {
  return await invoke<void>('plugin:gigi-p2p|group_delete', { groupId })
}

/**
 * Update a group
 * @param groupId - The group ID
 * @param groupName - Optional new group name
 * @param joined - Optional joined status
 */
export async function groupUpdate(
  groupId: string,
  groupName?: string,
  joined?: boolean
): Promise<GroupInfo> {
  return await invoke<GroupInfo>('plugin:gigi-p2p|group_update', {
    groupId,
    groupName,
    joined,
  })
}

// Types

export interface AccountInfo {
  address: string
  peer_id: string
  group_id: string
  name: string
}

export interface LoginResult {
  account_info: AccountInfo
  private_key: string
}

export interface GroupInfo {
  group_id: string
  name: string
  joined: boolean
  created_at: number
}
