

Gigi：基于 P2P 网络的社交应用
====

本项目为一个基于 P2P 网络的社交应用，结合了 Rust 和 TypeScript/React 技术栈，使用 Tauri 进行多端开发。

## 项目结构概览

- `apps/gigi-mobile`：Gigi 移动端应用。
- `pkgs/gigi-messaging`：一个 Tauri 后端，用于实现基于 libp2p 的消息传递功能。

## 开发与构建

### Gigi 主应用

#### 移动开发

1. 确保你已安装 Rust 和 Tauri CLI。
2. 进入 `apps/gigi-mobile` 目录
    - bun install
    - bun run tauri dev, or
    - bun run tauri android dev
3. 构建
    - bun run tauri build, or
    - bun run tauri android build

## 服务条款

详见 `apps/gigi-mobile/src/assets/TermsOfUse.md`。

## 如何贡献

欢迎贡献代码，提交 Issue 或 Pull Request。请遵循项目代码规范，并确保提交的代码通过测试。

## 许可证

本项目遵循 MIT 许可证。详见根目录下的 `LICENSE` 文件。
