import { useState, useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
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
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    if (location.state && location.state.tab) {
      setActiveTab(location.state.tab);
    }
  }, [location.state]);

  return (
    <div className="min-h-screen flex flex-col bg-base-200">
      <Dock value={activeTab} onValueChange={setActiveTab}>
        {/* 首页内容 */}
        <Dock.Content value="chat" className="mb-6">
          <ChatList onChatSelect={(chatId) => navigate(`/chat/${chatId}`)} />
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
              <div className="flex flex-col items-center text-lg">
                <Chat className="dock-icon" />
                <span className="dock-text">聊天</span>
              </div>
            </div>
          </Dock.Trigger>

          <Dock.Trigger value="contact" className="dock-item">
            <div className="dock-label">
              <div className="flex flex-col items-center text-lg">
                <Contact className="dock-icon" />
                <span className="dock-text">通讯录</span>
              </div>
            </div>
          </Dock.Trigger>

          <Dock.Trigger value="discover" className="dock-item">
            <div className="dock-label">
              <div className="flex flex-col items-center text-lg">
                <Discover className="dock-icon" />
                <span className="dock-text">发现</span>
              </div>
            </div>
          </Dock.Trigger>

          <Dock.Trigger value="me" className="dock-item">
            <div className="dock-label">
              <div className="flex flex-col items-center text-lg">
                <Me className="dock-icon" />
                <span className="dock-text">我</span>
              </div>
            </div>
          </Dock.Trigger>
        </Dock.List>
      </Dock>
    </div>
  );
}
