import { useEffect, useRef } from 'react'
import { MessagingEvents } from '@/utils/messaging'

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
  // Use a ref to track if we've already set up listeners to prevent duplicates
  const listenersSetupRef = useRef(false)

  useEffect(() => {
    // Prevent setting up listeners multiple times
    if (listenersSetupRef.current) {
      return
    }
    listenersSetupRef.current = true

    // Listen to various P2P events
    const events = [
      { name: 'peer-id-changed', type: 'success' as const },
      { name: 'public-key-changed', type: 'success' as const },
      { name: 'peer-discovered', type: 'info' as const },
      { name: 'peer-connected', type: 'success' as const },
      { name: 'peer-disconnected', type: 'warning' as const },
      { name: 'peer-expired', type: 'info' as const },
      { name: 'direct-message', type: 'info' as const },
      { name: 'group-message', type: 'info' as const },
      { name: 'file-share-request', type: 'info' as const },
      { name: 'download-progress', type: 'info' as const },
      { name: 'download-completed', type: 'success' as const },
      { name: 'download-failed', type: 'error' as const },
      { name: 'nickname-changed', type: 'info' as const },
      { name: 'nickname-updated', type: 'info' as const },
      { name: 'config-changed', type: 'info' as const },
      { name: 'p2p-error', type: 'error' as const },
    ]

    // Set up event listeners
    events.forEach(({ name, type }) => {
      MessagingEvents.on(name, data => {
        addLog(name, data, type)
      })
    })

    // Add initial log only if logs are empty (first time mounting)
    if (logs.length === 0) {
      addLog('system', { message: 'P2P Event Logger Started' }, 'success')
    }

    // Immediately request current state on component mount
    const fetchCurrentState = async () => {
      try {
        const { MessagingClient } = await import('@/utils/messaging')

        // First, emit current state to trigger any missed events
        await MessagingClient.emitCurrentState()

        // Then get current peers to populate logs if they exist
        const peers = await MessagingClient.getPeers()

        if (peers.length > 0) {
          peers.forEach(peer => {
            addLog(
              'peer-connected',
              {
                peer_id: peer.id,
                nickname: peer.nickname,
              },
              'success'
            )
          })
        }

        addLog('test-peers', { count: peers.length, peers }, 'info')
      } catch (error) {
        console.error('❌ Error fetching current state:', error)
        addLog('test-error', { error: error.message }, 'error')
      }
    }

    // Execute immediately to catch up on any events that were missed
    fetchCurrentState()

    // Also try after delays to handle race conditions
    setTimeout(fetchCurrentState, 1000)
    setTimeout(fetchCurrentState, 3000)

    // Cleanup
    return () => {
      events.forEach(({ name }) => {
        // Note: MessagingEvents doesn't have an off method that works properly
        // For now, we'll just keep the listeners active
      })
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
        <h3 className="text-lg font-semibold">P2P Event Logs</h3>
        <div className="flex gap-2 flex-wrap">
          <button
            onClick={() => {
              // Trigger a test event
              addLog(
                'test-event',
                { message: 'This is a test event to verify logging works' },
                'info'
              )
            }}
            className="px-3 py-1 text-sm bg-blue-200 hover:bg-blue-300 rounded"
          >
            Test Event
          </button>
          <button
            onClick={async () => {
              // Test P2P functionality by getting peers
              try {
                const { MessagingClient } = await import('@/utils/messaging')
                const peers = await MessagingClient.getPeers()
                addLog(
                  'peers-check',
                  {
                    count: peers.length,
                    peers: peers.map(p => ({ id: p.id, nickname: p.nickname })),
                  },
                  'info'
                )
              } catch (error) {
                addLog('peers-check-error', { error: error.message }, 'error')
              }
            }}
            className="px-3 py-1 text-sm bg-green-200 hover:bg-green-300 rounded"
          >
            Check Peers
          </button>
          <button
            onClick={async () => {
              // Test getting peer ID
              try {
                const { MessagingClient } = await import('@/utils/messaging')
                const peerId = await MessagingClient.getPeerId()
                addLog('peer-id-check', { peerId }, 'success')
              } catch (error) {
                addLog('peer-id-error', { error: error.message }, 'error')
              }
            }}
            className="px-3 py-1 text-sm bg-purple-200 hover:bg-purple-300 rounded"
          >
            Get Peer ID
          </button>
          <button
            onClick={clearLogs}
            className="px-3 py-1 text-sm bg-gray-200 hover:bg-gray-300 rounded"
          >
            Clear Logs
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto bg-gray-50 rounded p-2 min-h-0">
        {logs.length === 0 ? (
          <p className="text-gray-500 text-center py-8">
            No logs yet. Events will appear here.
          </p>
        ) : (
          <div className="space-y-2">
            {logs.map(log => (
              <div
                key={log.id}
                className="bg-white p-3 rounded shadow-sm border-l-4 border-blue-400"
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
