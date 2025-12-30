import { useEffect } from 'react'

interface LogEntry {
  id: string
  timestamp: string
  event: string
  data: any
  type: 'info' | 'success' | 'warning' | 'error'
}

interface P2PLogsProps {
  logs: LogEntry[]
  addLog: (event: string, data: any, type?: LogEntry['type']) => void
  clearLogs: () => void
}

export default function P2PLogs({ logs, addLog, clearLogs }: P2PLogsProps) {
  // Add initial log only if logs are empty (first time mounting)
  useEffect(() => {
    if (logs.length === 0) {
      addLog(
        'system',
        { message: 'Android Output Directory Logger Started' },
        'success'
      )
    }
  }, [logs.length, addLog])

  const getLogColor = (type: LogEntry['type']) => {
    switch (type) {
      case 'success':
        return 'text-green-600'
      case 'warning':
        return 'text-yellow-600'
      case 'error':
        return 'text-red-600'
      default:
        return 'text-blue-600'
    }
  }

  const formatData = (data: any) => {
    try {
      return JSON.stringify(data, null, 2)
    } catch {
      return String(data)
    }
  }

  return (
    <div className="flex flex-col h-full p-4">
      <div className="flex justify-between items-center mb-4 flex-shrink-0">
        <h3 className="text-lg font-semibold">Android File Sharing Logs</h3>
        <button
          onClick={clearLogs}
          className="px-3 py-1 text-sm bg-gray-200 hover:bg-gray-300 rounded"
        >
          Clear Logs
        </button>
      </div>

      <div className="flex-1 overflow-y-auto bg-gray-50 rounded p-2 min-h-0">
        {logs.length === 0 ? (
          <p className="text-gray-500 text-center py-8">
            No file sharing logs yet. Content URI resolution events will appear
            here.
          </p>
        ) : (
          <div className="space-y-2">
            {logs.map(log => (
              <div
                key={log.id}
                className="bg-white p-3 rounded shadow-sm border-l-4 border-green-400"
              >
                <div className="flex justify-between items-start mb-2">
                  <span
                    className={`font-mono text-sm font-semibold ${getLogColor(log.type)}`}
                  >
                    {log.event}
                  </span>
                  <span className="text-xs text-gray-500">
                    {new Date(log.timestamp).toLocaleTimeString()}
                  </span>
                </div>
                <pre className="text-xs bg-gray-100 p-2 rounded overflow-x-auto">
                  {formatData(log.data)}
                </pre>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
