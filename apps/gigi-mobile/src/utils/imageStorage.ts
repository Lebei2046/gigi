import { db, type Image } from '../models/db'

// Store image
export async function storeImage(
  imageId: string,
  imageFile: File
): Promise<string> {
  try {
    const imageObject = {
      id: imageId,
      data: imageFile,
      type: imageFile.type,
      createdAt: new Date(),
    }

    await db.images.put(imageObject)
    return imageId
  } catch (error) {
    console.error('Error storing image:', error)
    throw error
  }
}

// Get image Blob URL
export async function getImageUrl(imageId: string): Promise<string> {
  try {
    const imageObject = await db.images.get(imageId)
    if (imageObject) {
      return URL.createObjectURL(imageObject.data)
    }
    throw new Error('Image not found')
  } catch (error) {
    console.error('Error retrieving image:', error)
    throw error
  }
}

// Get image Blob data
export async function getImageBlob(imageId: string): Promise<Blob> {
  try {
    const imageObject = await db.images.get(imageId)
    if (imageObject) {
      return imageObject.data
    }
    throw new Error('Image not found')
  } catch (error) {
    console.error('Error retrieving image blob:', error)
    throw error
  }
}

// Delete image
export async function deleteImage(imageId: string): Promise<void> {
  try {
    await db.images.delete(imageId)
  } catch (error) {
    console.error('Error deleting image:', error)
    throw error
  }
}

// Batch delete images
export async function deleteImages(imageIds: string[]): Promise<void> {
  try {
    await db.images.bulkDelete(imageIds)
  } catch (error) {
    console.error('Error deleting images:', error)
    throw error
  }
}

// Clean up image URL (avoid memory leaks)
export function revokeImageUrl(url: string): void {
  URL.revokeObjectURL(url)
}

// Check if image exists
export async function imageExists(imageId: string): Promise<boolean> {
  try {
    const count = await db.images.where('id').equals(imageId).count()
    return count > 0
  } catch (error) {
    console.error('Error checking image existence:', error)
    return false
  }
}

// Get all image info (excluding actual Blob data)
export async function getAllImageInfo(): Promise<Omit<Image, 'data'>[]> {
  try {
    const images = await db.images.toArray()
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    return images.map(({ data, ...info }) => info)
  } catch (error) {
    console.error('Error retrieving image info:', error)
    throw error
  }
}

// Store user avatar
export async function storeAvatar(
  address: string,
  imageFile: File
): Promise<string> {
  try {
    // Generate unique image ID
    const imageId = `avatar_${address}_${Date.now()}`

    // Store image
    await storeImage(imageId, imageFile)

    // Save or update user avatar association record
    const now = new Date()
    await db.avatars.put({
      id: address, // Use address as unique identifier
      imageId,
      createdAt: now,
      updatedAt: now,
    })

    return imageId
  } catch (error) {
    console.error('Error storing avatar:', error)
    throw error
  }
}

// Get user avatar URL
export async function getAvatarUrl(address: string): Promise<string | null> {
  try {
    // Find user avatar record
    const avatarRecord = await db.avatars.get(address)
    if (avatarRecord) {
      // Get image URL
      return await getImageUrl(avatarRecord.imageId)
    }
    return null
  } catch (error) {
    console.error('Error retrieving avatar:', error)
    return null
  }
}

// Delete user avatar
export async function deleteAvatar(address: string): Promise<void> {
  try {
    // Find user avatar record
    const avatarRecord = await db.avatars.get(address)
    if (avatarRecord) {
      // Delete associated image
      await deleteImage(avatarRecord.imageId)
      // Delete avatar record
      await db.avatars.delete(address)
    }
  } catch (error) {
    console.error('Error deleting avatar:', error)
    throw error
  }
}
