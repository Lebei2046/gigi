import { useState, useEffect, useRef } from 'react';
import { FiMic, FiSmile, FiPlusCircle } from 'react-icons/fi';
import { MdKeyboard } from 'react-icons/md';
import EmojiCard from './EmojiCard';
import PlusCard from './PlusCard';

interface InputBarProps {
  onSend: (content: string) => void;
  onCardHeightChange: (height: number) => void;
  onInputHeightChange: (height: number) => void;
}

const InputBar = ({
  onSend,
  onCardHeightChange,
  onInputHeightChange,
}: InputBarProps) => {
  const [inputText, setInputText] = useState('');
  const [isRecording, setIsRecording] = useState(false);
  const [activeCard, setActiveCard] = useState<'emoji' | 'plus' | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const barRef = useRef<HTMLDivElement>(null);
  const cardContainerRef = useRef<HTMLDivElement>(null);
  const ignoreFocus = useRef(false);

  // 更新输入区高度
  useEffect(() => {
    if (barRef.current) {
      const height = barRef.current.clientHeight;
      onInputHeightChange(height);
    }
  }, [isRecording, activeCard, onInputHeightChange]);

  // 更新卡片高度
  useEffect(() => {
    const updateCardHeight = () => {
      if (cardContainerRef.current) {
        const height = activeCard ? cardContainerRef.current.scrollHeight : 0;
        onCardHeightChange(height);

        // 滚动优化 - 延迟获取更精确的高度
        if (activeCard) {
          setTimeout(() => {
            const finalHeight = cardContainerRef.current?.scrollHeight || 0;
            if (finalHeight > height) {
              onCardHeightChange(finalHeight);
            }
          }, 20);
        }
      }
    };

    updateCardHeight();
  }, [activeCard, onCardHeightChange]);

  // 关闭所有卡片
  const closeAllCards = () => {
    setActiveCard(null);
  };

  // 切换语音输入模式
  const handleVoiceClick = () => {
    setIsRecording(!isRecording);
    closeAllCards();
    if (isRecording) {
      onSend('[语音消息]');
    }
  };

  // 切换表情包卡片
  const toggleEmojiCard = () => {
    if (activeCard === 'emoji') {
      closeAllCards();
    } else {
      ignoreFocus.current = true;
      setActiveCard('emoji');
      setTimeout(() => {
        inputRef.current?.focus();
        setTimeout(() => {
          ignoreFocus.current = false;
        }, 100);
      }, 10);
    }
  };

  // 切换功能卡片
  const togglePlusCard = () => {
    if (activeCard === 'plus') {
      closeAllCards();
    } else {
      setActiveCard('plus');
    }
  };

  // 处理表情选择
  const handleSelectEmoji = (emoji: string) => {
    setInputText((prev) => prev + emoji);
    ignoreFocus.current = true;
    setTimeout(() => {
      inputRef.current?.focus();
      ignoreFocus.current = false;
    }, 50);
  };

  // 发送消息
  const handleSubmit = () => {
    if (inputText.trim()) {
      onSend(inputText);
      setInputText('');
      closeAllCards();
    }
  };

  // 修复1: 增强点击外部关闭卡片逻辑
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      // 确保点击目标有效
      const target = e.target as HTMLElement;
      if (!barRef.current || !target) return;

      // 检查点击是否在输入区内部
      const isClickInside = barRef.current.contains(target);

      // 点击在外部则关闭卡片
      if (!isClickInside) {
        closeAllCards();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // 处理输入框获得焦点事件
  const handleInputFocus = () => {
    if (!ignoreFocus.current && activeCard) {
      closeAllCards();
    }
  };

  return (
    <div ref={barRef} className="bg-gray-100 h-full">
      {/* 输入工具栏 */}
      <div className="p-3 h-full">
        <div className="flex items-center h-full">
          <button
            type="button"
            onClick={handleVoiceClick}
            className="p-2 transition-colors hover:bg-gray-200 rounded-full"
          >
            {isRecording ? <MdKeyboard size={24} /> : <FiMic size={24} />}
          </button>

          {isRecording ? (
            <button
              type="button"
              className="flex-1 bg-white py-3 px-4 rounded-full text-center hover:bg-gray-200"
            >
              按住说话
            </button>
          ) : (
            <input
              ref={inputRef}
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              className="flex-1 border-0 rounded-full py-2 px-4 mx-2 focus:ring-2 focus:ring-blue-300"
              placeholder="输入消息"
              onKeyDown={(e) => e.key === 'Enter' && handleSubmit()}
              onFocus={handleInputFocus}
            />
          )}

          <button
            type="button"
            onClick={toggleEmojiCard}
            className={`p-2 transition-colors rounded-full ${activeCard === 'emoji' ? 'bg-gray-200' : 'hover:bg-gray-200'}`}
          >
            {activeCard === 'emoji' ? (
              <MdKeyboard size={24} />
            ) : (
              <FiSmile size={24} />
            )}
          </button>

          <button
            type="button"
            onClick={togglePlusCard}
            className={`p-2 transition-colors rounded-full ${activeCard === 'plus' ? 'bg-gray-200' : 'hover:bg-gray-200'}`}
          >
            <FiPlusCircle size={24} />
          </button>
        </div>
      </div>

      {/* 卡片区域 - 从下方弹出 */}
      <div
        ref={cardContainerRef}
        className={`absolute top-full left-0 right-0 overflow-hidden transition-all duration-300 bg-gray-100 ${activeCard ? 'translate-y-0' : 'translate-y-full'
          }`}
      >
        {activeCard === 'emoji' && (
          <div className="bg-white px-2 pb-2 pt-1 shadow-lg">
            {/* 修复2: 确保表情卡片事件正确处理 */}
            <EmojiCard
              onSelect={(emoji) => {
                console.log('选择表情:', emoji); // 添加调试日志
                handleSelectEmoji(emoji);
              }}
              onSend={() => {
                console.log('点击发送按钮'); // 添加调试日志
                handleSubmit();
                closeAllCards();
              }}
            />
          </div>
        )}
        {activeCard === 'plus' && (
          <div className="bg-white px-2 pb-2 pt-1 shadow-lg">
            {/* 修复3: 确保加号卡片事件正确处理 */}
            <PlusCard
              onSelect={(action) => {
                console.log('选择加号功能:', action); // 确保日志打印
                // 保留原有的回调关闭逻辑
                closeAllCards();
              }}
            />
          </div>
        )}
      </div>
    </div>
  );
};

export default InputBar;
