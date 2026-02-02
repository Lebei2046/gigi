/* eslint-disable @typescript-eslint/no-explicit-any */
import {
  createSlice,
  type PayloadAction,
  createAsyncThunk,
} from '@reduxjs/toolkit'
import { invoke } from '@tauri-apps/api/core'

// Define types for files
interface FileItem {
  id: string
  name: string
  type: string
  size: number
  createdAt: Date
  sender: string
  isFavorite: boolean
  status: 'completed' | 'downloading' | 'error'
  progress?: number
}

interface DownloadItem {
  id: string
  name: string
  type: string
  size: number
  progress: number
  speed?: string
  timeRemaining?: string
  status: 'downloading' | 'completed' | 'error'
}

interface FileState {
  files: FileItem[]
  downloads: DownloadItem[]
  favorites: string[] // Array of favorite file IDs
  searchTerm: string
  selectedCategory: 'all' | 'favorites' | 'recent' | 'downloads'
  status: 'idle' | 'loading' | 'succeeded' | 'failed'
  error: string | null
}

const initialState: FileState = {
  files: [],
  downloads: [],
  favorites: [],
  searchTerm: '',
  selectedCategory: 'all',
  status: 'idle',
  error: null,
}

// Async thunks

// Fetch all files
export const fetchFilesAsync = createAsyncThunk(
  'files/fetchFiles',
  async (_, { rejectWithValue }) => {
    try {
      // This would be replaced with actual Tauri command
      // const response = await invoke('get_all_files')
      // return response as FileItem[]

      // Mock data for now
      return [
        {
          id: '1',
          name: 'VacationPhoto.jpg',
          type: 'image',
          size: 2048576,
          createdAt: new Date('2024-01-15T10:30:00'),
          sender: 'Alice',
          isFavorite: true,
          status: 'completed',
        },
        {
          id: '2',
          name: 'ProjectPlan.pdf',
          type: 'pdf',
          size: 1536000,
          createdAt: new Date('2024-01-14T15:45:00'),
          sender: 'Bob',
          isFavorite: false,
          status: 'completed',
        },
        {
          id: '3',
          name: 'MeetingNotes.docx',
          type: 'document',
          size: 512000,
          createdAt: new Date('2024-01-13T09:20:00'),
          sender: 'Charlie',
          isFavorite: true,
          status: 'completed',
        },
        {
          id: '4',
          name: 'Budget2024.xlsx',
          type: 'spreadsheet',
          size: 896000,
          createdAt: new Date('2024-01-12T14:10:00'),
          sender: 'Dave',
          isFavorite: false,
          status: 'completed',
        },
        {
          id: '5',
          name: 'NewProjectPresentation.pdf',
          type: 'pdf',
          size: 3072000,
          createdAt: new Date('2024-01-11T11:50:00'),
          sender: 'Eve',
          isFavorite: false,
          status: 'downloading',
          progress: 75,
        },
      ]
    } catch (error) {
      return rejectWithValue('Failed to fetch files')
    }
  }
)

// Fetch downloads
export const fetchDownloadsAsync = createAsyncThunk(
  'files/fetchDownloads',
  async (_, { rejectWithValue }) => {
    try {
      // This would be replaced with actual Tauri command
      // const response = await invoke('get_downloads')
      // return response as DownloadItem[]

      // Mock data for now
      return [
        {
          id: '6',
          name: 'BigFile.zip',
          type: 'archive',
          size: 10485760,
          progress: 60,
          speed: '1.2 MB/s',
          timeRemaining: '2m 30s',
          status: 'downloading',
        },
        {
          id: '7',
          name: 'VacationVideo.mp4',
          type: 'video',
          size: 52428800,
          progress: 100,
          status: 'completed',
        },
        {
          id: '8',
          name: 'MusicAlbum.rar',
          type: 'archive',
          size: 20971520,
          progress: 100,
          status: 'error',
        },
      ]
    } catch (error) {
      return rejectWithValue('Failed to fetch downloads')
    }
  }
)

// Toggle favorite status
export const toggleFavoriteAsync = createAsyncThunk(
  'files/toggleFavorite',
  async (fileId: string, { rejectWithValue, getState }) => {
    try {
      // This would be replaced with actual Tauri command
      // await invoke('toggle_favorite', { fileId })

      // Return fileId for reducer to handle
      return fileId
    } catch (error) {
      return rejectWithValue('Failed to toggle favorite')
    }
  }
)

// Start download
export const startDownloadAsync = createAsyncThunk(
  'files/startDownload',
  async (fileId: string, { rejectWithValue, getState }) => {
    try {
      // This would be replaced with actual Tauri command
      // const response = await invoke('start_download', { fileId })
      // return response as DownloadItem

      // Mock data for now
      return {
        id: fileId,
        name: 'NewDownloadFile.pdf',
        type: 'pdf',
        size: 5242880,
        progress: 0,
        speed: '0 MB/s',
        timeRemaining: 'Calculating...',
        status: 'downloading',
      }
    } catch (error) {
      return rejectWithValue('Failed to start download')
    }
  }
)

const fileSlice = createSlice({
  name: 'files',
  initialState,
  reducers: {
    setSearchTerm: (state, action: PayloadAction<string>) => {
      state.searchTerm = action.payload
    },
    setSelectedCategory: (
      state,
      action: PayloadAction<'all' | 'favorites' | 'recent' | 'downloads'>
    ) => {
      state.selectedCategory = action.payload
    },
    updateDownloadProgress: (
      state,
      action: PayloadAction<{
        id: string
        progress: number
        speed?: string
        timeRemaining?: string
      }>
    ) => {
      const download = state.downloads.find(d => d.id === action.payload.id)
      if (download) {
        download.progress = action.payload.progress
        download.speed = action.payload.speed
        download.timeRemaining = action.payload.timeRemaining
        if (download.progress === 100) {
          download.status = 'completed'
        }
      }
    },
    cancelDownload: (state, action: PayloadAction<string>) => {
      const download = state.downloads.find(d => d.id === action.payload)
      if (download) {
        download.status = 'error'
      }
    },
  },
  extraReducers: builder => {
    // Fetch files
    builder.addCase(fetchFilesAsync.pending, state => {
      state.status = 'loading'
    })
    builder.addCase(fetchFilesAsync.fulfilled, (state, action) => {
      state.status = 'succeeded'
      state.files = action.payload
      state.favorites = action.payload
        .filter(file => file.isFavorite)
        .map(file => file.id)
    })
    builder.addCase(fetchFilesAsync.rejected, (state, action) => {
      state.status = 'failed'
      state.error = action.payload as string
    })

    // Fetch downloads
    builder.addCase(fetchDownloadsAsync.pending, state => {
      state.status = 'loading'
    })
    builder.addCase(fetchDownloadsAsync.fulfilled, (state, action) => {
      state.status = 'succeeded'
      state.downloads = action.payload
    })
    builder.addCase(fetchDownloadsAsync.rejected, (state, action) => {
      state.status = 'failed'
      state.error = action.payload as string
    })

    // Toggle favorite
    builder.addCase(toggleFavoriteAsync.fulfilled, (state, action) => {
      const fileId = action.payload
      const file = state.files.find(f => f.id === fileId)
      if (file) {
        file.isFavorite = !file.isFavorite
        if (file.isFavorite) {
          state.favorites.push(fileId)
        } else {
          state.favorites = state.favorites.filter(id => id !== fileId)
        }
      }
    })

    // Start download
    builder.addCase(startDownloadAsync.fulfilled, (state, action) => {
      state.downloads.push(action.payload)
    })
  },
})

export const {
  setSearchTerm,
  setSelectedCategory,
  updateDownloadProgress,
  cancelDownload,
} = fileSlice.actions
export default fileSlice.reducer
