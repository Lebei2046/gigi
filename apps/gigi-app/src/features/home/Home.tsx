import { useState } from 'react';
import {
  FaComment as Chat,
  FaAddressBook as Contact,
  FaCompass as Discover,
  FaUser as Me
} from 'react-icons/fa';

import Dock from './components/Dock';
import ChatList from "./views/ChatList";
import ContactList from "./views/ContactList";
import MePage from "./views/MePage";
import DiscoverPage from "./views/DiscoverPage";

export default function Home() {
  const [activeTab, setActiveTab] = useState('chat');

  return (
    <div className="min-h-screen flex flex-col bg-base-200">
      {/* 使用 Dock 组件包裹所有内容 */}
      <Dock value={activeTab} onValueChange={setActiveTab}>
        {/* 首页内容 */}
        <Dock.Content value="chat" className="mb-6">
          <ChatList />
        </Dock.Content>

        {/* 搜索内容 */}
        <Dock.Content value="contact" className="mb-6">
          <ContactList />
        </Dock.Content>

        {/* 音乐内容 */}
        <Dock.Content value="discover" className="mb-6">
          <DiscoverPage />
        </Dock.Content>

        {/* 个人资料内容 */}
        <Dock.Content value="me" className="mb-6">
          <MePage />
        </Dock.Content>

        {/* 固定在底部的 Dock 导航栏 */}
        <Dock.List className="dock-bottom p-2 bg-base-100 bg-opacity-90 backdrop-blur-sm shadow-lg z-50">
          <Dock.Trigger value="chat" className="dock-item">
            <div className="dock-label">
              <Chat className="dock-icon text-xl" />
              <span className="dock-text text-lg">聊天</span>
            </div>
          </Dock.Trigger>

          <Dock.Trigger value="contact" className="dock-item">
            <div className="dock-label">
              <div className="flex flex-col items-center">
                <Contact className="dock-icon text-xl" />
                <span className="dock-text text-lg">通讯录</span>
              </div>
            </div>
          </Dock.Trigger>

          <Dock.Trigger value="discover" className="dock-item">
            <div className="dock-label">
              <div className="flex flex-col items-center">
                <Discover className="dock-icon text-xl" />
                <span className="dock-text text-lg">发现</span>
              </div>
            </div>
          </Dock.Trigger>

          <Dock.Trigger value="me" className="dock-item">
            <div className="dock-label">
              <div className="flex flex-col items-center">
                <Me className="dock-icon text-xl" />
                <span className="dock-text text-lg">我</span>
              </div>
            </div>
          </Dock.Trigger>
        </Dock.List>
      </Dock>
    </div>
  );
}
