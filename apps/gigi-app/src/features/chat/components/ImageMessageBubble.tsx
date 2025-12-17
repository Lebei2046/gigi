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
    let currentImageUrl: string | null = null;

    const loadImage = async () => {
      try {
        setLoading(true);
        setError(null);

        const url = await getImageUrl(imageId);

        // Only update state if component is still mounted
        if (isMounted) {
          currentImageUrl = url;
          setImageUrl(url);
        }
      } catch (err) {
        // Only update state if component is still mounted
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

    // Cleanup function
    return () => {
      isMounted = false;
      // Only revoke current image URL when component unmounts
      if (currentImageUrl) {
        revokeImageUrl(currentImageUrl);
      }
    };
  }, [imageId]); // Remove imageUrl dependency, only depend on imageId

  // Fixed container height to reduce layout jumping
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
