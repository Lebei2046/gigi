import React from 'react';

interface EmojiCardProps {
  onSelect: (emoji: string) => void;
  onSend: () => void;
}

const EmojiCard: React.FC<EmojiCardProps> = ({ onSelect, onSend }) => {
  const emojiCategories = [
    {
      name: '表情',
      emojis: ['😀', '😂', '😍', '😎', '🥰', '😜', '🥺', '😢'],
    },
    {
      name: '符号',
      emojis: ['❤️', '🔥', '✨', '⭐', '💯', '🎉', '👍', '👏'],
    },
    {
      name: '动物',
      emojis: ['🐶', '🐱', '🐭', '🐰', '🦊', '🐻', '🐼', '🐨'],
    },
    {
      name: '食物',
      emojis: ['🍎', '🍐', '🍊', '🍋', '🍌', '🍉', '🍇', '🍓'],
    },
  ];

  return (
    <div className="bg-white rounded-lg border p-3 max-h-80 overflow-auto">
      <div className="sticky top-0 bg-white z-10 pb-2">
        <div className="flex border-b">
          {emojiCategories.map((category) => (
            <button
              key={category.name}
              type="button"
              className="px-3 py-2 text-sm hover:bg-gray-100"
            >
              {category.name}
            </button>
          ))}
        </div>
      </div>

      {emojiCategories.map((category) => (
        <div key={category.name} className="mb-4 pt-3">
          <div className="text-gray-500 text-sm font-medium pl-2 mb-2">
            {category.name}
          </div>
          <div className="grid grid-cols-8 gap-1">
            {category.emojis.map((emoji) => (
              <button
                key={emoji}
                type="button"
                // 添加详细事件处理
                onClick={() => {
                  onSelect(emoji);
                }}
                className="text-2xl p-1 hover:bg-gray-200 rounded transition-colors"
              >
                {emoji}
              </button>
            ))}
          </div>
        </div>
      ))}

      <div className="flex justify-end p-2">
        <button
          type="button"
          // 添加详细事件处理
          onClick={() => {
            onSend();
          }}
          className="bg-green-500 hover:bg-green-600 text-white px-4 py-2 rounded-full transition-colors"
        >
          发送
        </button>
      </div>
    </div>
  );
};

export default EmojiCard;
