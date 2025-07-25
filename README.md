# Gigi：基于 P2P 网络的社交应用

Gigi 是一款基于 P2P 网络的社交应用，旨在为用户提供去中心化的社交体验。前端项目为 `gigi-app`。

## 项目结构概览

- **前端 (gigi-app)**
  - 使用 React 框架开发，采用 TypeScript。
  - 包含主要功能模块如聊天、首页、登录、注册等。
  - 使用 Vite 构建工具，配置文件位于 `vite.config.ts`。
  - 项目入口文件为 `src/main.tsx`，UI 根组件为 `src/App.tsx`。

- **功能模块**
  - **聊天功能 (Chat)**
    - 包含聊天面板 (`ChatPanel`)、消息面板 (`MessagePanel`)、输入栏 (`InputBar`)、顶部栏 (`TopBar`) 等组件。
    - 消息气泡 (`MessageBubble`) 和表情卡片 (`EmojiCard`) 提供丰富的交互体验。
    - 消息操作卡片 (`MessageActionCard`) 支持对消息的进一步操作。

  - **首页功能 (Home)**
    - 包含聊天列表 (`ChatList`)、联系人列表 (`ContactList`)、发现页面 (`DiscoverPage`) 和个人页面 (`MePage`)。
    - 提供头像 (`Avatar`)、聊天列表项 (`ChatListItem`)、联系人列表项 (`ContactListItem`) 等基础组件。
    - Dock 组件 (`Dock`) 提供底部导航功能，包含列表 (`List`)、触发器 (`Trigger`) 和内容 (`Content`) 子组件。

  - **登录与注册 (Login/Signup)**
    - 登录模块包含解锁 (`Unlock`) 和重置账户 (`ResetAccount`) 功能。
    - 注册模块包含多步骤注册流程 (`Signup`)，提供助记词确认 (`MnemonicConfirm`)、助记词显示 (`MnemonicDisplay`)、助记词输入 (`MnemonicInput`) 等组件。
    - 使用 `SignupContext` 提供注册状态管理。

- **数据模型**
  - 包含聊天 (`Chat`)、联系人 (`Contact`)、消息 (`Message`) 等数据模型。
  - 使用 `db.ts` 提供本地数据库支持。

- **工具与辅助**
  - `utils/crypto.ts` 提供加密相关工具函数。
  - `utils/storage.ts` 提供本地存储管理。
  - 使用 Redux 管理全局状态 (`store/authSlice.ts`)。

## 开发与构建

- 使用 Vite 构建工具，配置文件为 `vite.config.ts`。
- 使用 TypeScript，配置文件为 `tsconfig.json`。
- 使用 ESLint 进行代码规范，配置文件为 `eslint.config.js`。

## 服务条款

请参阅 `src/assets/TermsOfUse.md` 了解 Gigi 社交应用的服务条款。

## 如何贡献

欢迎贡献代码和反馈意见！请遵循项目的代码规范和提交指南。