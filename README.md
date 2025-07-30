

Gigi：基于 P2P 网络的社交应用
====

本项目为一个基于 P2P 网络的社交应用，结合了 Rust 和 TypeScript 技术栈，使用 Tauri 开发桌面客户端，并包含一个用于测试的 Tauri 插件应用。

## 项目结构概览

- `apps/gigi-rust`：包含 Gigi 桌面应用的 Rust 部分。
- `apps/gigi-ts`：包含 Gigi 桌面应用的前端 TypeScript 部分，使用 React 框架。
- `packages/tauri-plugin-libp2p-messaging`：一个 Tauri 插件，用于实现基于 libp2p 的消息传递功能。
- `packages/tauri-plugin-test-app`：用于测试插件功能的简单 Tauri 应用，采用 Svelte 框架。

## 开发与构建

### Gigi 主应用

#### 开发环境

1. 确保你已安装 Rust 和 Tauri CLI。
2. 安装依赖：进入 `apps/gigi-ts` 目录并运行 `npm install`。
3. 启动前端开发服务器：`npm run dev`。
4. 运行桌面应用：`npm run tauri dev`。

#### 构建

- 构建发布版本：`npm run tauri build`。

#### 服务条款

详见 `apps/gigi-ts/src/assets/TermsOfUse.md`。

## 如何贡献

欢迎贡献代码，提交 Issue 或 Pull Request。请遵循项目代码规范，并确保提交的代码通过测试。

## 许可证

本项目遵循 MIT 许可证。详见根目录下的 `LICENSE` 文件。