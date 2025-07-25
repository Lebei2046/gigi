import { db, type Image } from '../models/db';

// 存储图片
export async function storeImage(imageId: string, imageFile: File): Promise<string> {
  try {
    const imageObject = {
      id: imageId,
      data: imageFile,
      type: imageFile.type,
      createdAt: new Date()
    };

    await db.images.put(imageObject);
    return imageId;
  } catch (error) {
    console.error('Error storing image:', error);
    throw error;
  }
}

// 获取图片 Blob URL
export async function getImageUrl(imageId: string): Promise<string> {
  try {
    const imageObject = await db.images.get(imageId);
    if (imageObject) {
      return URL.createObjectURL(imageObject.data);
    }
    throw new Error('Image not found');
  } catch (error) {
    console.error('Error retrieving image:', error);
    throw error;
  }
}

// 获取图片 Blob 数据
export async function getImageBlob(imageId: string): Promise<Blob> {
  try {
    const imageObject = await db.images.get(imageId);
    if (imageObject) {
      return imageObject.data;
    }
    throw new Error('Image not found');
  } catch (error) {
    console.error('Error retrieving image blob:', error);
    throw error;
  }
}

// 删除图片
export async function deleteImage(imageId: string): Promise<void> {
  try {
    await db.images.delete(imageId);
  } catch (error) {
    console.error('Error deleting image:', error);
    throw error;
  }
}

// 批量删除图片
export async function deleteImages(imageIds: string[]): Promise<void> {
  try {
    await db.images.bulkDelete(imageIds);
  } catch (error) {
    console.error('Error deleting images:', error);
    throw error;
  }
}

// 清理图片 URL（避免内存泄漏）
export function revokeImageUrl(url: string): void {
  URL.revokeObjectURL(url);
}

// 检查图片是否存在
export async function imageExists(imageId: string): Promise<boolean> {
  try {
    const count = await db.images.where('id').equals(imageId).count();
    return count > 0;
  } catch (error) {
    console.error('Error checking image existence:', error);
    return false;
  }
}

// 获取所有图片信息（不包括实际的 Blob 数据）
export async function getAllImageInfo(): Promise<Omit<Image, 'data'>[]> {
  try {
    const images = await db.images.toArray();
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    return images.map(({ data, ...info }) => info);
  } catch (error) {
    console.error('Error retrieving image info:', error);
    throw error;
  }
}

// 存储用户头像
export async function storeAvatar(address: string, imageFile: File): Promise<string> {
  try {
    // 生成唯一的图片ID
    const imageId = `avatar_${address}_${Date.now()}`;

    // 存储图片
    await storeImage(imageId, imageFile);

    // 保存或更新用户头像关联记录
    const now = new Date();
    await db.avatars.put({
      id: address, // 使用address作为唯一标识
      imageId,
      createdAt: now,
      updatedAt: now
    });

    return imageId;
  } catch (error) {
    console.error('Error storing avatar:', error);
    throw error;
  }
}

// 获取用户头像URL
export async function getAvatarUrl(address: string): Promise<string | null> {
  try {
    // 查找用户头像记录
    const avatarRecord = await db.avatars.get(address);
    if (avatarRecord) {
      // 获取图片URL
      return await getImageUrl(avatarRecord.imageId);
    }
    return null;
  } catch (error) {
    console.error('Error retrieving avatar:', error);
    return null;
  }
}

// 删除用户头像
export async function deleteAvatar(address: string): Promise<void> {
  try {
    // 查找用户头像记录
    const avatarRecord = await db.avatars.get(address);
    if (avatarRecord) {
      // 删除关联的图片
      await deleteImage(avatarRecord.imageId);
      // 删除头像记录
      await db.avatars.delete(address);
    }
  } catch (error) {
    console.error('Error deleting avatar:', error);
    throw error;
  }
}
