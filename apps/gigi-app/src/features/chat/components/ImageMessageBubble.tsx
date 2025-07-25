import { useState, useEffect } from 'react';
import { getImageUrl, revokeImageUrl } from '../../../utils/imageStorage';

interface ImageMessageBubbleProps {
  imageId: string;
}

const ImageMessageBubble = ({ imageId }: ImageMessageBubbleProps) => {
  const [imageUrl, setImageUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;

    const loadImage = async () => {
      try {
        setLoading(true);
        setError(null);

        const url = await getImageUrl(imageId);

        // 只有在组件仍然挂载时才更新状态
        if (isMounted) {
          setImageUrl(url);
        }
      } catch (err) {
        // 只有在组件仍然挂载时才更新状态
        if (isMounted) {
          setError(err instanceof Error ? err.message : 'Failed to load image');
        }
      } finally {
        if (isMounted) {
          setLoading(false);
        }
      }
    };

    loadImage();

    // 清理函数
    return () => {
      isMounted = false;
      if (imageUrl) {
        revokeImageUrl(imageUrl);
      }
    };
  }, [imageId]); // 依赖数组只包含 imageId

  // 固定容器高度以减少布局跳动
  if (loading) {
    return (
      <div className="bg-gray-200 rounded-lg p-4 flex items-center justify-center" style={{ width: '200px', height: '200px' }}>
        <div className="text-gray-500">Loading image...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-100 text-red-700 rounded-lg p-4 flex items-center justify-center" style={{ width: '200px', height: '200px' }}>
        Failed to load image: {error}
      </div>
    );
  }

  if (!imageUrl) {
    return (
      <div className="bg-gray-100 rounded-lg p-4 flex items-center justify-center" style={{ width: '200px', height: '200px' }}>
        No image available
      </div>
    );
  }

  return (
    <img
      src={imageUrl}
      alt="Chat image"
      className="max-w-xs max-h-60 rounded-lg object-contain"
      onError={() => setError('Failed to display image')}
      style={{ width: '200px', height: '200px', objectFit: 'cover' }}
    />
  );
};

export default ImageMessageBubble;
