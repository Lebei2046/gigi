import { useState } from 'react';
import Dock from './components/Dock';
import { FaHome, FaSearch, FaMusic, FaUser } from 'react-icons/fa';

export default function Home() {
  const [activeTab, setActiveTab] = useState('home');

  return (
    <div className="min-h-screen flex flex-col bg-base-200">
      {/* 主内容区域 */}
      <main className="flex-1 p-4 overflow-y-auto">
        <div className="max-w-4xl mx-auto">
          <h1 className="text-3xl font-bold text-base-content mb-6">Dock 组件演示</h1>
          <p className="text-base-content mb-8">
            使用 DaisyUI 的 dock 样式实现的底部导航栏
          </p>

          {/* 使用 Dock 组件包裹所有内容 */}
          <Dock value={activeTab} onValueChange={setActiveTab}>
            {/* 首页内容 */}
            <Dock.Content value="home" className="mb-6">
              <div className="card bg-base-100 shadow-xl">
                <div className="card-body">
                  <div className="flex items-center gap-4 mb-4">
                    <div className="bg-primary p-3 rounded-full text-primary-content">
                      <FaHome className="text-2xl" />
                    </div>
                    <h2 className="text-2xl font-bold">首页内容</h2>
                  </div>
                  <p>
                    欢迎来到应用程序首页！这里展示您的主要信息和概览。
                    您可以查看最新动态、通知和个性化推荐内容。
                  </p>
                </div>
              </div>
            </Dock.Content>

            {/* 搜索内容 */}
            <Dock.Content value="search" className="mb-6">
              <div className="card bg-base-100 shadow-xl">
                <div className="card-body">
                  <div className="flex items-center gap-4 mb-4">
                    <div className="bg-secondary p-3 rounded-full text-secondary-content">
                      <FaSearch className="text-2xl" />
                    </div>
                    <h2 className="text-2xl font-bold">搜索功能</h2>
                  </div>
                  <p>
                    使用我们的高级搜索功能查找您需要的内容。
                    支持多种筛选条件和关键字搜索，帮助您快速找到所需信息。
                  </p>
                </div>
              </div>
            </Dock.Content>

            {/* 音乐内容 */}
            <Dock.Content value="music" className="mb-6">
              <div className="card bg-base-100 shadow-xl">
                <div className="card-body">
                  <div className="flex items-center gap-4 mb-4">
                    <div className="bg-accent p-3 rounded-full text-accent-content">
                      <FaMusic className="text-2xl" />
                    </div>
                    <h2 className="text-2xl font-bold">音乐播放器</h2>
                  </div>
                  <p>
                    播放您喜欢的音乐，创建个性化播放列表。
                    享受高品质音频体验和无缝切换功能。
                  </p>
                </div>
              </div>
            </Dock.Content>

            {/* 个人资料内容 */}
            <Dock.Content value="profile" className="mb-6">
              <div className="card bg-base-100 shadow-xl">
                <div className="card-body">
                  <div className="flex items-center gap-4 mb-4">
                    <div className="bg-neutral p-3 rounded-full text-neutral-content">
                      <FaUser className="text-2xl" />
                    </div>
                    <h2 className="text-2xl font-bold">个人资料</h2>
                  </div>
                  <p>
                    管理您的个人资料、设置和偏好。
                    更新您的个人信息和账户安全设置，自定义您的用户体验。
                  </p>
                </div>
              </div>
            </Dock.Content>

            {/* 固定在底部的 Dock 导航栏 */}
            <Dock.List className="dock-bottom p-2 bg-base-100 bg-opacity-90 backdrop-blur-sm shadow-lg z-50">
              <Dock.Trigger value="home" className="dock-item">
                <div className="dock-label">
                  <FaHome className="dock-icon" />
                  <span className="dock-text">首页</span>
                </div>
              </Dock.Trigger>

              <Dock.Trigger value="search" className="dock-item">
                <div className="dock-label">
                  <FaSearch className="dock-icon" />
                  <span className="dock-text">搜索</span>
                </div>
              </Dock.Trigger>

              <Dock.Trigger value="music" className="dock-item">
                <div className="dock-label">
                  <FaMusic className="dock-icon" />
                  <span className="dock-text">音乐</span>
                </div>
              </Dock.Trigger>

              <Dock.Trigger value="profile" className="dock-item">
                <div className="dock-label">
                  <FaUser className="dock-icon" />
                  <span className="dock-text">个人</span>
                </div>
              </Dock.Trigger>
            </Dock.List>
          </Dock>
        </div>
      </main>
    </div>
  );
}
