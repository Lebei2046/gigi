import { useEffect, useState } from 'react'
import type { Peer } from '@/utils/messaging'
import { MessagingClient, MessagingEvents } from '@/utils/messaging'
import { useAppDispatch } from '@/store'
import { addLog } from '@/store/logsSlice'

export default function Chat() {
  const [peers, setPeers] = useState<Peer[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const dispatch = useAppDispatch()

  // Error boundary for this component
  try {
    useEffect(() => {
      console.log('Chat component mounting...')
      dispatch(
        addLog({
          event: 'chat_loaded',
          data: 'Chat component loaded successfully',
          type: 'info',
        })
      )

      const loadPeers = async () => {
        try {
          console.log('Loading real peers...')

          // Add timeout to prevent hanging
          const timeoutPromise = new Promise((_, reject) => {
            setTimeout(() => reject(new Error('Timeout loading peers')), 5000)
          })

          const peersPromise = MessagingClient.getPeers()
          const connectedPeers = (await Promise.race([
            peersPromise,
            timeoutPromise,
          ])) as Peer[]

          console.log('Connected peers:', connectedPeers)
          setPeers(connectedPeers)
          dispatch(
            addLog({
              event: 'peers_loaded',
              data: `Loaded ${connectedPeers.length} connected peers`,
              type: 'info',
            })
          )
        } catch (error) {
          console.error('Failed to load peers:', error)
          setError(`Failed to load peers: ${error}`)
          setPeers([]) // Ensure we have an empty array on error
          dispatch(
            addLog({
              event: 'peers_load_error',
              data: `Failed to load peers: ${error}`,
              type: 'error',
            })
          )
        } finally {
          setLoading(false)
        }
      }

      loadPeers()

      // Set up periodic polling as fallback for real-time updates
      const pollInterval = setInterval(async () => {
        try {
          console.log('Polling for peers...')
          const currentPeers = await MessagingClient.getPeers()
          console.log('Current peers from poll:', currentPeers)
          setPeers(currentPeers)
        } catch (error) {
          console.error('Error polling peers:', error)
        }
      }, 3000) // Poll every 3 seconds

      // Also try event listeners with debugging
      try {
        console.log('Setting up event listeners...')

        const handlePeerConnected = (peer: any) => {
          console.log('🔗 Peer connected event received:', peer)
          try {
            if (peer && peer.id && peer.nickname) {
              setPeers(prev => {
                const exists = prev.find(p => p.id === peer.id)
                if (!exists) {
                  console.log('Adding new peer to state:', peer)
                  return [...prev, peer]
                }
                return prev
              })
              dispatch(
                addLog({
                  event: 'peer_connected',
                  data: `Peer connected: ${peer.nickname} (${peer.id})`,
                  type: 'info',
                })
              )
            }
          } catch (error) {
            console.error('Error in handlePeerConnected:', error)
          }
        }

        const handlePeerDisconnected = (peer: any) => {
          console.log('❌ Peer disconnected event received:', peer)
          try {
            if (peer && peer.id && peer.nickname) {
              setPeers(prev => prev.filter(p => p.id !== peer.id))
              dispatch(
                addLog({
                  event: 'peer_disconnected',
                  data: `Peer disconnected: ${peer.nickname} (${peer.id})`,
                  type: 'info',
                })
              )
            }
          } catch (error) {
            console.error('Error in handlePeerDisconnected:', error)
          }
        }

        // Register event listeners
        MessagingEvents.on('peer-connected', handlePeerConnected)
        MessagingEvents.on('peer-disconnected', handlePeerDisconnected)
        console.log('Event listeners registered')

        // Clean up on unmount
        return () => {
          clearInterval(pollInterval)
          try {
            MessagingEvents.off('peer-connected', handlePeerConnected)
            MessagingEvents.off('peer-disconnected', handlePeerDisconnected)
            console.log('Event listeners cleaned up')
          } catch (error) {
            console.error('Error cleaning up event listeners:', error)
          }
        }
      } catch (error) {
        console.error('Error setting up event listeners:', error)
        return () => clearInterval(pollInterval)
      }
    }, [dispatch])

    if (error) {
      return (
        <div className="flex flex-col items-center justify-center h-full p-4">
          <p className="text-red-500">Error: {error}</p>
          <button
            onClick={() => window.location.reload()}
            className="mt-4 px-4 py-2 bg-blue-500 text-white rounded"
          >
            Reload
          </button>
        </div>
      )
    }

    if (loading) {
      return (
        <div className="flex items-center justify-center h-full">
          <p className="text-gray-500">Loading peers...</p>
        </div>
      )
    }

    return (
      <div className="flex flex-col h-full p-4">
        <h2 className="text-xl font-semibold mb-4">Peers</h2>

        {peers.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <p className="text-gray-500">
              No peers found. Make sure other devices are running Gigi on the
              same network.
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {peers.map(peer => (
              <div
                key={peer.id}
                className="p-3 bg-green-50 border border-green-200 rounded-lg"
              >
                <div className="font-medium text-green-800">
                  {peer.nickname}
                </div>
                <div className="text-sm text-gray-600">{peer.id}</div>
                {peer.capabilities.length > 0 && (
                  <div className="text-xs text-gray-500 mt-1">
                    {peer.capabilities.join(', ')}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    )
  } catch (error) {
    console.error('Chat component error:', error)
    return (
      <div className="flex items-center justify-center h-full p-4">
        <div className="text-center">
          <p className="text-red-500">Chat component error</p>
          <p className="text-sm text-gray-500">{String(error)}</p>
          <button
            onClick={() => window.location.reload()}
            className="mt-4 px-4 py-2 bg-blue-500 text-white rounded"
          >
            Reload
          </button>
        </div>
      </div>
    )
  }
}
