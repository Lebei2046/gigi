明白了！前端已经准备好了，现在需要将前端的 signup 和 login 流程迁移到使用后端命令。

让我先查看前端的相关代码，了解当前的实现：
现在我了解了前端的实现。让我创建一个详细的迁移计划，将前端从 IndexedDB 迁移到使用后端 Tauri commands。

# 前端迁移计划：从 IndexedDB 到后端 Commands

## 📊 当前架构分析

### 现有前端实现

**Signup 流程** (`SignupContext.tsx`):
1. 前端生成助记词
2. 前端派生密钥 (`peer_id`, `address`)
3. 前端加密助记词并保存到 IndexedDB (`setStorageItem('gigi', ...)`)
4. 前端保存群组到 IndexedDB (`db.groups.add(...)`)
5. 跳转到登录页面

**Login 流程** (`authSlice.ts`):
1. 从 IndexedDB 读取加密数据 (`getStorageItem('gigi')`)
2. 前端解密助记词
3. 前端验证地址
4. 前端派生私钥并初始化 P2P
5. 更新 Redux 状态

---

## 🎯 迁移目标

将数据存储从 IndexedDB 迁移到后端，前端改为调用 Tauri commands。

---

## 📝 迁移步骤

### **阶段 1: 创建 Tauri Command 调用工具**

创建 `/src/utils/tauriCommands.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core'
import { hexToBytes } from './crypto'

// ===== Auth Commands =====

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

/**
 * 创建账户
 * @param mnemonic - 助记词（空格分隔的字符串）
 * @param password - 密码
 * @param name - 用户昵称
 * @param groupName - 可选的群组名称
 */
export async function authSignup(
  mnemonic: string,
  password: string,
  name: string,
  groupName?: string | null
): Promise<AccountInfo> {
  return await invoke<AccountInfo>('auth_signup', {
    mnemonic,
    password,
    name,
    groupName,
  })
}

/**
 * 登录
 * @param password - 密码
 */
export async function authLogin(password: string): Promise<LoginResult> {
  return await invoke<LoginResult>('auth_login', { password })
}

/**
 * 获取账户信息
 */
export async function authGetAccountInfo(): Promise<AccountInfo | null> {
  return await invoke<AccountInfo | null>('auth_get_account_info')
}

/**
 * 检查账户是否存在
 */
export async function authHasAccount(): Promise<boolean> {
  return await invoke<boolean>('auth_has_account')
}

/**
 * 修改密码
 */
export async function authChangePassword(
  oldPassword: string,
  newPassword: string
): Promise<void> {
  return await invoke<void>('auth_change_password', {
    oldPassword: oldPassword,
    newPassword: newPassword,
  })
}

/**
 * 删除账户
 */
export async function authDeleteAccount(): Promise<void> {
  return await invoke<void>('auth_delete_account')
}

/**
 * 验证密码
 */
export async function authVerifyPassword(password: string): Promise<boolean> {
  return await invoke<boolean>('auth_verify_password', { password })
}

// ===== Group Commands =====

export interface GroupInfo {
  group_id: string
  name: string
  joined: boolean
  created_at: number // timestamp in milliseconds
}

/**
 * 添加或更新群组
 */
export async function groupAddOrUpdate(
  groupId: string,
  name: string,
  joined: boolean
): Promise<void> {
  return await invoke<void>('group_add_or_update', {
    groupId,
    name,
    joined,
  })
}

/**
 * 获取群组信息
 */
export async function groupGet(groupId: string): Promise<GroupInfo | null> {
  return await invoke<GroupInfo | null>('group_get', { groupId })
}

/**
 * 获取所有群组
 */
export async function groupGetAll(): Promise<GroupInfo[]> {
  return await invoke<GroupInfo[]>('group_get_all')
}

/**
 * 获取已加入的群组
 */
export async function groupGetJoined(): Promise<GroupInfo[]> {
  return await invoke<GroupInfo[]>('group_get_joined')
}

/**
 * 更新群组加入状态
 */
export async function groupUpdateJoinStatus(
  groupId: string,
  joined: boolean
): Promise<boolean> {
  return await invoke<boolean>('group_update_join_status', {
    groupId,
    joined,
  })
}

/**
 * 更新群组名称
 */
export async function groupUpdateName(
  groupId: string,
  name: string
): Promise<boolean> {
  return await invoke<boolean>('group_update_name', { groupId, name })
}

/**
 * 删除群组
 */
export async function groupDelete(groupId: string): Promise<boolean> {
  return await invoke<boolean>('group_delete', { groupId })
}

/**
 * 检查群组是否存在
 */
export async function groupExists(groupId: string): Promise<boolean> {
  return await invoke<boolean>('group_exists', { groupId })
}

/**
 * 检查用户是否已加入群组
 */
export async function groupIsJoined(groupId: string): Promise<boolean> {
  return await invoke<boolean>('group_is_joined', { groupId })
}

/**
 * 清空所有群组
 */
export async function groupClearAll(): Promise<number> {
  return await invoke<number>('group_clear_all')
}

/**
 * 获取群组数量
 */
export async function groupCount(): Promise<number> {
  return await invoke<number>('group_count')
}

/**
 * 获取已加入群组数量
 */
export async function groupCountJoined(): Promise<number> {
  return await invoke<number>('group_count_joined')
}

// ===== P2P Messaging Commands =====

/**
 * 初始化 P2P 客户端（使用私钥）
 */
export async function messagingInitializeWithKey(
  privateKey: string, // hex string
  nickname: string
): Promise<string> {
  const privateKeyBytes = hexToBytes(privateKey)
  return await invoke<string>('messaging_initialize_with_key', {
    privateKey: privateKeyBytes,
    nickname,
  })
}
```

---

### **阶段 2: 更新 SignupContext**

更新 `/src/features/signup/context/SignupContext.tsx`:

```typescript
import React, { createContext, useContext, useReducer } from 'react'
import {
  initialState,
  signupReducer,
  type SignupAction,
  type SignupState,
} from './signupReducer'
import {
  authSignup,
  type AccountInfo,
} from '@/utils/tauriCommands'

type SignupContextType = {
  state: SignupState
  dispatch: React.Dispatch<SignupAction>
  saveAccountInfo: () => Promise<AccountInfo>
}

const SignupContext = createContext<SignupContextType | undefined>(undefined)

export function SignupProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(signupReducer, initialState)

  const saveAccountInfo = async (): Promise<AccountInfo> {
    // 将助记词数组转换为字符串
    const mnemonicString = state.mnemonic.join(' ')
    
    // 调用后端 auth_signup 命令
    const accountInfo: AccountInfo = await authSignup(
      mnemonicString,
      state.password,
      state.name,
      state.createGroup ? state.groupName : null
    )

    // Update state after successful save
    dispatch({ 
      type: 'ACCOUNT_INFO_SAVED', 
      payload: { 
        address: accountInfo.address, 
        peerId: accountInfo.peer_id 
      } 
    })

    return accountInfo
  }

  return (
    <SignupContext.Provider
      value={{ state, dispatch, saveAccountInfo }}
    >
      {children}
    </SignupContext.Provider>
  )
}

// eslint-disable-next-line react-refresh/only-export-components
export function useSignupContext() {
  const context = useContext(SignupContext)
  if (!context) {
    throw new Error('useSignupContext must be used within a SignupProvider')
  }
  return context
}
```

---

### **阶段 3: 更新 SignupFinish 组件**

更新 `/src/features/signup/pages/SignupFinish.tsx`:

```typescript
import { useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { useSignupContext } from '../context/SignupContext'

export default function SignupFinish() {
  const navigate = useNavigate()
  const {
    state: { address, peerId, name, createGroup, groupName },
    saveAccountInfo,
  } = useSignupContext()

  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')
  const [savedAddress, setSavedAddress] = useState('')
  const [savedPeerId, setSavedPeerId] = useState('')

  useEffect(() => {
    const saveInfo = async () => {
      setIsLoading(true)
      setError('')

      try {
        // 调用后端保存账户信息（包含群组创建逻辑）
        const accountInfo = await saveAccountInfo()
        
        setSavedAddress(accountInfo.address)
        setSavedPeerId(accountInfo.peer_id)
      } catch (err) {
        console.error('Failed to save account:', err)
        setError(err instanceof Error ? err.message : 'Failed to create account')
      } finally {
        setIsLoading(false)
      }
    }
    saveInfo()
  }, [saveAccountInfo])

  const handleLogin = async () => {
    navigate('/login')
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Creating your account...</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center max-w-md">
          <div className="mx-auto w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mb-4">
            <svg className="w-8 h-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
          </div>
          <h1 className="text-2xl font-bold text-gray-900 mb-2">
            Account Creation Failed
          </h1>
          <p className="text-gray-600 mb-6">{error}</p>
          <Button onClick={handleLogin} variant="outline">
            Go Back
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="text-center space-y-2">
        <div className="mx-auto w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mb-4">
          <svg
            className="w-8 h-8 text-green-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M5 13l4 4L19 7"
            ></path>
          </svg>
        </div>
        <h1 className="text-2xl font-bold text-gray-900">
          Account Created Successfully!
        </h1>
        <p className="text-gray-600">Your new account is ready to use</p>
      </div>

      <div className="bg-white rounded-xl border border-gray-200 shadow-sm p-6 space-y-4">
        <h2 className="text-lg font-semibold text-gray-900 flex items-center gap-2">
          <span className="w-6 h-6 bg-blue-100 rounded-full flex items-center justify-center text-xs font-bold text-blue-600">
            i
          </span>
          Account Details
        </h2>

        <div className="space-y-3">
          <div className="flex justify-between items-center py-2 border-b border-gray-100">
            <span className="text-sm font-medium text-gray-600">
              Account Name
            </span>
            <span className="text-sm font-medium text-gray-900">{name}</span>
          </div>

          <div className="flex justify-between items-center py-2 border-b border-gray-100">
            <span className="text-sm font-medium text-gray-600">
              Account Address
            </span>
            <span className="text-xs font-mono text-gray-500 bg-gray-100 px-2 py-1 rounded">
              {savedAddress?.slice(0, 8)}...{savedAddress?.slice(-8)}
            </span>
          </div>

          <div className="flex justify-between items-center py-2 border-b border-gray-100">
            <span className="text-sm font-medium text-gray-600">Peer ID</span>
            <span className="text-xs font-mono text-gray-500 bg-gray-100 px-2 py-1 rounded">
              {savedPeerId?.slice(0, 8)}...{savedPeerId?.slice(-8)}
            </span>
          </div>
        </div>

        {createGroup && groupName.trim() && (
          <div className="mt-4 bg-green-50 border border-green-200 rounded-lg p-4">
            <h3 className="text-sm font-semibold text-green-800 flex items-center gap-2 mb-2">
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                ></path>
              </svg>
              Group Created Successfully
            </h3>
            <p className="text-sm text-green-700 font-medium">
              {groupName.trim()}
            </p>
            <p className="text-xs text-green-600 mt-1">
              Your group has been created and is ready to use!
            </p>
          </div>
        )}
      </div>

      <Button
        onClick={handleLogin}
        className="w-full py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-all duration-200"
      >
        Continue to Login
      </Button>
    </div>
  )
}
```

---

### **阶段 4: 更新 authSlice**

更新 `/src/store/authSlice.ts`:

```typescript
/* eslint-disable @typescript-eslint/no-explicit-any */
import { createSlice, type PayloadAction } from '@reduxjs/toolkit'
import {
  authHasAccount,
  authLogin,
  authGetAccountInfo,
  authDeleteAccount,
  messagingInitializeWithKey,
  type AccountInfo,
  type LoginResult,
} from '../utils/tauriCommands'

type AuthState = {
  status: 'unregistered' | 'unauthenticated' | 'authenticated'
  accountInfo: AccountInfo | null
  password: string | null
  peerId: string | null
  groups: any[]
  error: string | null
}

const initialState: AuthState = {
  status: 'unregistered',
  accountInfo: null,
  password: null,
  peerId: null,
  groups: [],
  error: null,
}

const authSlice = createSlice({
  name: 'auth',
  initialState,
  reducers: {
    clearAuth: state => {
      state.status = 'unauthenticated'
      state.accountInfo = null
      state.password = null
      state.peerId = null
    },
    setUnregistered: state => {
      state.status = 'unregistered'
      state.accountInfo = null
    },
    loginSuccess: (
      state,
      action: PayloadAction<{ password: string; peerId: string }>
    ) => {
      state.status = 'authenticated'
      state.password = action.payload.password
      state.peerId = action.payload.peerId
      state.error = null
    },
    setAccountInfo: (state, action: PayloadAction<AccountInfo>) => {
      state.accountInfo = action.payload
    },
    setGroups: (state, action: PayloadAction<any[]>) => {
      state.groups = action.payload
    },
    resetState: state => {
      state.status = 'unregistered'
      state.accountInfo = null
      state.password = null
      state.peerId = null
      state.groups = []
      state.error = null
    },
    setError: (state, action: PayloadAction<string>) => {
      state.error = action.payload
    },
  },
})

// Async action to check if account exists and load account info
export const initAuth = () => async (dispatch: any) => {
  try {
    const hasAccount = await authHasAccount()

    if (!hasAccount) {
      dispatch(setUnregistered())
    } else {
      const accountInfo = await authGetAccountInfo()
      if (accountInfo) {
        dispatch(setAccountInfo(accountInfo))
        dispatch({ type: 'auth/setAccountInfo', payload: accountInfo })
      }
    }
  } catch (error) {
    console.error('Failed to initialize auth:', error)
    dispatch(setUnregistered())
  }
}

// Async action for login with P2P initialization
export const loginWithP2P =
  (password: string) => async (dispatch: any, getState: any) => {
    try {
      // 调用后端 auth_login
      const loginResult: LoginResult = await authLogin(password)

      const { account_info, private_key } = loginResult

      // 保存账户信息到 Redux
      dispatch(setAccountInfo(account_info))

      // 初始化 P2P 客户端
      const peerId = await messagingInitializeWithKey(private_key, account_info.name)

      dispatch(loginSuccess({ password, peerId }))

      return { success: true, peerId, accountInfo: account_info }
    } catch (error) {
      console.error('Login error:', error)
      const errorMessage =
        error instanceof Error ? error.message : 'Login failed'
      dispatch(setError(errorMessage))
      return { success: false, error: errorMessage }
    }
  }

// Async action to delete account
export const deleteAccount = () => async (dispatch: any) => {
  try {
    await authDeleteAccount()
    dispatch(resetState())
    return { success: true }
  } catch (error) {
    console.error('Failed to delete account:', error)
    const errorMessage =
      error instanceof Error ? error.message : 'Failed to delete account'
    return { success: false, error: errorMessage }
  }
}

export const {
  clearAuth,
  setUnregistered,
  loginSuccess,
  setAccountInfo,
  setGroups,
  resetState,
  setError,
} = authSlice.actions

export default authSlice.reducer
```

---

### **阶段 5: 更新 ResetAccount 组件**

如果需要更新 `/src/features/signin/ResetAccount.tsx`:

```typescript
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { deleteAccount } from '@/store/authSlice'
import { useAppDispatch } from '@/store'

export default function ResetAccount() {
  const navigate = useNavigate()
  const dispatch = useAppDispatch()
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState('')

  const handleReset = async () => {
    if (
      !confirm(
        'Are you sure you want to delete your account? This action cannot be undone and will delete all your data.'
      )
    ) {
      return
    }

    setIsLoading(true)
    setError('')

    try {
      const result = await dispatch(deleteAccount())

      if (result?.success) {
        navigate('/signup')
      } else if (result?.error) {
        setError(result.error)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to reset account')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-red-50 to-gray-50 px-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">
            Reset Account
          </h1>
          <p className="text-gray-600">
            Delete your account and all associated data
          </p>
        </div>

        <div className="bg-white rounded-2xl shadow-lg border border-gray-100 p-6 space-y-6">
          {error && (
            <div className="bg-red-50 border border-red-200 rounded-lg p-4">
              <p className="text-red-600 text-sm">{error}</p>
            </div>
          )}

          <div className="space-y-4">
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
              <h3 className="text-sm font-semibold text-yellow-800 mb-2">
                ⚠️ Warning
              </h3>
              <p className="text-sm text-yellow-700">
                This action cannot be undone. All your data including:
              </p>
              <ul className="list-disc list-inside text-sm text-yellow-700 mt-2 ml-2">
                <li>Account information</li>
                <li>Encrypted mnemonic</li>
                <li>Groups</li>
                <li>Messages</li>
                <li>All other stored data</li>
              </ul>
            </div>

            <Button
              onClick={handleReset}
              disabled={isLoading}
              className="w-full py-3 bg-red-600 hover:bg-red-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-white font-medium rounded-lg transition-all duration-200"
            >
              {isLoading ? 'Deleting...' : 'Delete Account'}
            </Button>

            <Button
              onClick={() => navigate('/login')}
              variant="outline"
              className="w-full"
            >
              Cancel
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
```

---

### **阶段 6: 清理不再使用的代码**

删除或注释以下文件/代码：

1. `/src/utils/settingStorage.ts` - 不再需要 IndexedDB 存储
2. `/src/models/db.ts` - 不再需要 IndexedDB 数据库
3. `/src/utils/crypto.ts` - 保留 `hexToBytes`，其他加密/派生函数可以移除（由后端处理）
4. 从 `SignupContext.tsx` 移除 `saveGroupInfo` 函数（已整合到后端 `auth_signup`）
5. 从 `authSlice.ts` 移除 `loadAuthData`（改为 `initAuth`）

---

## ✅ 验收标准

- [ ] Signup 流程成功创建账户到后端
- [ ] Login 流程成功从后端读取并验证账户
- [ ] P2P 客户端正确初始化
- [ ] 群组正确创建和读取
- [ ] 错误处理完善
- [ ] 不再使用 IndexedDB 存储敏感数据
- [ ] 所有 Tauri commands 正确调用

---

## 📝 注意事项

1. **向后兼容性**: 迁移期间，确保旧用户数据可以迁移或提示用户重新注册
2. **错误处理**: 所有 Tauri 调用都需要 try-catch 和用户友好的错误提示
3. **加载状态**: 所有异步操作都需要加载状态指示器
4. **类型安全**: 使用 TypeScript 类型确保类型安全
5. **测试**: 在迁移后进行完整的端到端测试

---

这个迁移计划涵盖了所有必要的前端改动。准备好开始实施了吗？