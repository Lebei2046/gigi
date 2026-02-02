import { useState, useMemo, useEffect } from 'react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  FaDownload as DownloadIcon,
  FaUpload as UploadIcon,
  FaStar as StarIcon,
  FaClock as ClockIcon,
  FaFileImage as ImageIcon,
  FaFilePdf as PdfIcon,
  FaFileWord as WordIcon,
  FaFileExcel as ExcelIcon,
  FaFileAlt as FileIcon,
  FaSearch as SearchIcon,
} from 'react-icons/fa'
import { useAppSelector, useAppDispatch } from '@/store'
import {
  fetchFilesAsync,
  fetchDownloadsAsync,
  toggleFavoriteAsync,
  startDownloadAsync,
  setSearchTerm,
  setSelectedCategory,
  cancelDownload,
  updateDownloadProgress,
} from '@/store/fileSlice'

const Files = () => {
  const [activeTab, setActiveTab] = useState('all')
  const dispatch = useAppDispatch()
  const { files, downloads, searchTerm, status, error } = useAppSelector(
    state => state.files
  )
  const user = useAppSelector(state => state.auth.user)

  // Fetch files and downloads on component mount
  useEffect(() => {
    dispatch(fetchFilesAsync())
    dispatch(fetchDownloadsAsync())
  }, [dispatch])

  // Simulate download progress updates
  useEffect(() => {
    const interval = setInterval(() => {
      downloads.forEach(download => {
        if (download.status === 'downloading' && download.progress < 100) {
          const newProgress = download.progress + Math.random() * 5
          dispatch(
            updateDownloadProgress({
              id: download.id,
              progress: Math.min(100, newProgress),
              speed: `${(Math.random() * 2 + 0.5).toFixed(1)} MB/s`,
              timeRemaining: `${Math.floor(Math.random() * 5 + 1)}m ${Math.floor(Math.random() * 60)}s`,
            })
          )
        }
      })
    }, 1000)

    return () => clearInterval(interval)
  }, [downloads, dispatch])

  // Get file icon based on type
  const getFileIcon = (type: string) => {
    switch (type) {
      case 'image':
        return <ImageIcon className="w-6 h-6 text-blue-500" />
      case 'pdf':
        return <PdfIcon className="w-6 h-6 text-red-500" />
      case 'document':
        return <WordIcon className="w-6 h-6 text-blue-600" />
      case 'spreadsheet':
        return <ExcelIcon className="w-6 h-6 text-green-500" />
      default:
        return <FileIcon className="w-6 h-6 text-gray-500" />
    }
  }

  // Format file size
  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  // Filter files based on search term
  const filteredFiles = useMemo(() => {
    if (!searchTerm) return files
    return files.filter(file =>
      file.name.toLowerCase().includes(searchTerm.toLowerCase())
    )
  }, [files, searchTerm])

  // Get favorite files
  const favoriteFiles = useMemo(() => {
    return filteredFiles.filter(file => file.isFavorite)
  }, [filteredFiles])

  // Get recent files (last 7 days)
  const recentFiles = useMemo(() => {
    const weekAgo = new Date()
    weekAgo.setDate(weekAgo.getDate() - 7)
    return filteredFiles.filter(file => new Date(file.createdAt) >= weekAgo)
  }, [filteredFiles])

  return (
    <div className="flex flex-col h-full p-4">
      {/* Header */}
      <div className="flex justify-between items-center mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Files</h1>
          <p className="text-gray-500">Manage your shared files</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="icon">
            <UploadIcon className="w-5 h-5" />
          </Button>
          <Button variant="outline" size="icon">
            <DownloadIcon className="w-5 h-5" />
          </Button>
        </div>
      </div>

      {/* Search */}
      <div className="relative mb-6">
        <SearchIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
        <Input
          placeholder="Search files..."
          className="pl-10"
          value={searchTerm}
          onChange={e => dispatch(setSearchTerm(e.target.value))}
        />
      </div>

      {/* File Tabs */}
      <Tabs defaultValue="all" className="flex-1" onValueChange={setActiveTab}>
        <TabsList className="mb-4">
          <TabsTrigger value="all">All Files</TabsTrigger>
          <TabsTrigger value="favorites">Favorites</TabsTrigger>
          <TabsTrigger value="recent">Recent</TabsTrigger>
          <TabsTrigger value="downloads">Downloads</TabsTrigger>
        </TabsList>

        {/* All Files Tab */}
        <TabsContent
          value="all"
          className="h-[calc(100%-4rem)] overflow-y-auto"
        >
          <div className="grid grid-cols-2 gap-4">
            {filteredFiles.map(file => (
              <Card key={file.id} className="overflow-hidden">
                <CardContent className="p-4">
                  <div className="flex justify-between items-start mb-3">
                    {getFileIcon(file.type)}
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      onClick={() => dispatch(toggleFavoriteAsync(file.id))}
                    >
                      <StarIcon
                        className={`w-4 h-4 ${file.isFavorite ? 'text-yellow-500 fill-yellow-500' : 'text-gray-400'}`}
                      />
                    </Button>
                  </div>
                  <div className="mb-1">
                    <p className="font-medium text-sm truncate">{file.name}</p>
                    <p className="text-xs text-gray-500">{file.sender}</p>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-xs text-gray-500">
                      {formatFileSize(file.size)}
                    </span>
                    {file.status === 'downloading' && (
                      <div className="w-12 h-2 bg-gray-200 rounded-full overflow-hidden">
                        <div
                          className="h-full bg-blue-500 rounded-full"
                          style={{ width: `${file.progress}%` }}
                        />
                      </div>
                    )}
                  </div>
                  {file.status === 'completed' && (
                    <div className="mt-2">
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full"
                        onClick={() => dispatch(startDownloadAsync(file.id))}
                      >
                        <DownloadIcon className="w-3 h-3 mr-1" /> Download
                      </Button>
                    </div>
                  )}
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        {/* Favorites Tab */}
        <TabsContent
          value="favorites"
          className="h-[calc(100%-4rem)] overflow-y-auto"
        >
          <div className="grid grid-cols-2 gap-4">
            {favoriteFiles.map(file => (
              <Card key={file.id} className="overflow-hidden">
                <CardContent className="p-4">
                  <div className="flex justify-between items-start mb-3">
                    {getFileIcon(file.type)}
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      onClick={() => dispatch(toggleFavoriteAsync(file.id))}
                    >
                      <StarIcon className="w-4 h-4 text-yellow-500 fill-yellow-500" />
                    </Button>
                  </div>
                  <div className="mb-1">
                    <p className="font-medium text-sm truncate">{file.name}</p>
                    <p className="text-xs text-gray-500">{file.sender}</p>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-xs text-gray-500">
                      {formatFileSize(file.size)}
                    </span>
                  </div>
                  <div className="mt-2">
                    <Button
                      variant="outline"
                      size="sm"
                      className="w-full"
                      onClick={() => dispatch(startDownloadAsync(file.id))}
                    >
                      <DownloadIcon className="w-3 h-3 mr-1" /> Download
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        {/* Recent Tab */}
        <TabsContent
          value="recent"
          className="h-[calc(100%-4rem)] overflow-y-auto"
        >
          <div className="grid grid-cols-2 gap-4">
            {recentFiles.map(file => (
              <Card key={file.id} className="overflow-hidden">
                <CardContent className="p-4">
                  <div className="flex justify-between items-start mb-3">
                    {getFileIcon(file.type)}
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                      onClick={() => dispatch(toggleFavoriteAsync(file.id))}
                    >
                      <StarIcon
                        className={`w-4 h-4 ${file.isFavorite ? 'text-yellow-500 fill-yellow-500' : 'text-gray-400'}`}
                      />
                    </Button>
                  </div>
                  <div className="mb-1">
                    <p className="font-medium text-sm truncate">{file.name}</p>
                    <p className="text-xs text-gray-500">{file.sender}</p>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-xs text-gray-500">
                      {formatFileSize(file.size)}
                    </span>
                    <span className="text-xs text-gray-500">
                      {new Date(file.createdAt).toLocaleDateString()}
                    </span>
                  </div>
                  <div className="mt-2">
                    <Button
                      variant="outline"
                      size="sm"
                      className="w-full"
                      onClick={() => dispatch(startDownloadAsync(file.id))}
                    >
                      <DownloadIcon className="w-3 h-3 mr-1" /> Download
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        {/* Downloads Tab */}
        <TabsContent
          value="downloads"
          className="h-[calc(100%-4rem)] overflow-y-auto"
        >
          <div className="space-y-4">
            {downloads.map(download => (
              <Card key={download.id} className="overflow-hidden">
                <CardContent className="p-4">
                  <div className="flex justify-between items-start mb-3">
                    {getFileIcon(download.type)}
                    {download.status === 'downloading' && (
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        onClick={() => dispatch(cancelDownload(download.id))}
                      >
                        <DownloadIcon className="w-4 h-4 text-red-500" />
                      </Button>
                    )}
                  </div>
                  <div className="mb-2">
                    <p className="font-medium text-sm">{download.name}</p>
                    <p className="text-xs text-gray-500">
                      {formatFileSize(download.size)}
                    </p>
                  </div>
                  <div className="space-y-2">
                    <div className="w-full h-3 bg-gray-200 rounded-full overflow-hidden">
                      <div
                        className={`h-full rounded-full ${download.status === 'error' ? 'bg-red-500' : 'bg-blue-500'}`}
                        style={{ width: `${download.progress}%` }}
                      />
                    </div>
                    {download.status === 'downloading' && (
                      <div className="flex justify-between items-center text-xs text-gray-500">
                        <span>{download.progress}%</span>
                        <span>
                          {download.speed} · {download.timeRemaining}
                        </span>
                      </div>
                    )}
                    {download.status === 'completed' && (
                      <div className="text-xs text-green-500">Completed</div>
                    )}
                    {download.status === 'error' && (
                      <div className="text-xs text-red-500">
                        Download failed
                      </div>
                    )}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  )
}

export default Files
