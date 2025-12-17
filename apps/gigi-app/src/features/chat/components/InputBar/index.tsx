import { useState, useEffect, useRef } from 'react';
import { FiMic, FiSmile, FiPlusCircle } from 'react-icons/fi';
import { MdKeyboard } from 'react-icons/md';
import EmojiCard from './EmojiCard';
import PlusCard from './PlusCard';
import { storeImage } from '../../../../utils/imageStorage'; // New import

interface InputBarProps {
  onSend: (content: string) => void;
  onCardHeightChange: (height: number) => void;
  onInputHeightChange: (height: number) => void;
  onImageSend?: (imageId: string) => void; // New prop for sending images
}

const InputBar = ({
  onSend,
  onCardHeightChange,
  onInputHeightChange,
  onImageSend, // New prop
}: InputBarProps) => {
  const [inputText, setInputText] = useState('');
  const [isRecording, setIsRecording] = useState(false);
  const [activeCard, setActiveCard] = useState<'emoji' | 'plus' | null>(null);
  const [isImageUploading, setIsImageUploading] = useState(false); // New state
  const inputRef = useRef<HTMLInputElement>(null);
  const barRef = useRef<HTMLDivElement>(null);
  const cardContainerRef = useRef<HTMLDivElement>(null);
  const ignoreFocus = useRef(false);
  const fileInputRef = useRef<HTMLInputElement>(null); // New ref for file input

  // Update input area height
  useEffect(() => {
    if (barRef.current) {
      const height = barRef.current.clientHeight;
      onInputHeightChange(height);
    }
  }, [isRecording, activeCard, onInputHeightChange]);

  // Update card height
  useEffect(() => {
    const updateCardHeight = () => {
      if (cardContainerRef.current) {
        const height = activeCard ? cardContainerRef.current.scrollHeight : 0;
        onCardHeightChange(height);

        // Scroll optimization - delayed fetching of more accurate height
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

  // Close all cards
  const closeAllCards = () => {
    setActiveCard(null);
  };

  // Toggle voice input mode
  const handleVoiceClick = () => {
    setIsRecording(!isRecording);
    closeAllCards();
    if (isRecording) {
      onSend('[Voice message]');
    }
  };

  // Toggle emoji card
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

  // Toggle function card
  const togglePlusCard = () => {
    if (activeCard === 'plus') {
      closeAllCards();
    } else {
      setActiveCard('plus');
    }
  };

  // Handle image upload
  const handleImageUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file && file.type.startsWith('image/')) {
      try {
        setIsImageUploading(true);
        // Generate unique ID
        const imageId = `img_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

        // Store image
        await storeImage(imageId, file);

        // Notify parent component to send image message
        if (onImageSend) {
          onImageSend(imageId);
        }
      } catch (error) {
        console.error('Error uploading image:', error);
        alert('Image upload failed');
      } finally {
        setIsImageUploading(false);
        // Reset file input
        if (fileInputRef.current) {
          fileInputRef.current.value = '';
        }
        // Close card
        closeAllCards();
      }
    }
  };

  // Handle emoji selection
  const handleSelectEmoji = (emoji: string) => {
    setInputText((prev) => prev + emoji);
    ignoreFocus.current = true;
    setTimeout(() => {
      inputRef.current?.focus();
      ignoreFocus.current = false;
    }, 50);
  };

  // Send message
  const handleSubmit = () => {
    if (inputText.trim()) {
      onSend(inputText);
      setInputText('');
      closeAllCards();
    }
  };

  // Fix 1: Enhanced logic for closing card when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      // Ensure click target is valid
      const target = e.target as HTMLElement;
      if (!barRef.current || !target) return;

      // Check if click is inside input area
      const isClickInside = barRef.current.contains(target);

      // Close card if click is outside
      if (!isClickInside) {
        closeAllCards();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Handle input field focus event
  const handleInputFocus = () => {
    if (!ignoreFocus.current && activeCard) {
      closeAllCards();
    }
  };

  return (
    <div ref={barRef} className="bg-gray-100 h-full">
      {/* Hidden file input element */}
      <input
        type="file"
        ref={fileInputRef}
        accept="image/*"
        onChange={handleImageUpload}
        style={{ display: 'none' }}
        id="image-upload-input"
      />

      {/* Input toolbar */}
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
              Hold to speak
            </button>
          ) : (
            <input
              ref={inputRef}
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              className="flex-1 border-0 rounded-full py-2 px-4 mx-2 focus:ring-2 focus:ring-blue-300"
              placeholder="Enter message"
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
            disabled={isImageUploading} // Disable button when image is uploading
          >
            {isImageUploading ? (
              <div className="w-6 h-6 border-t-2 border-blue-500 rounded-full animate-spin"></div>
            ) : (
              <FiPlusCircle size={24} />
            )}
          </button>
        </div>
      </div>

      {/* Card area - pops up from bottom */}
      <div
        ref={cardContainerRef}
        className={`absolute top-full left-0 right-0 overflow-hidden transition-all duration-300 bg-gray-100 ${activeCard ? 'translate-y-0' : 'translate-y-full'
          }`}
      >
        {activeCard === 'emoji' && (
          <div className="bg-white px-2 pb-2 pt-1 shadow-lg">
            {/* Fix 2: Ensure emoji card events are handled correctly */}
            <EmojiCard
              onSelect={(emoji) => {
                handleSelectEmoji(emoji);
              }}
              onSend={() => {
                handleSubmit();
                closeAllCards();
              }}
            />
          </div>
        )}
        {activeCard === 'plus' && (
          <div className="bg-white px-2 pb-2 pt-1 shadow-lg">
            {/* Fix 3: Ensure plus card events are handled correctly */}
            <PlusCard
              onSelect={(action) => {
                // Execute different functions based on selected action
                switch (action) {
                  case 'Image':
                    // Trigger file selection
                    document.getElementById('image-upload-input')?.click();
                    break;
                  default:
                    // Other operations maintain original callback close logic
                    closeAllCards();
                }
              }}
            />
          </div>
        )}
      </div>
    </div>
  );
};

export default InputBar;
