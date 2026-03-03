import { useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getBootstrapNodes } from '../config/p2p'

/**
 * Hook to initialize P2P network configuration
 * 
 * This hook sets up the bootstrap nodes for Kademlia DHT discovery
 * when the app starts. It should be called early in the app lifecycle.
 */
export default function useInitP2P() {
  useEffect(() => {
    // Initialize P2P configuration with bootstrap nodes
    const initP2P = async () => {
      try {
        // Get bootstrap nodes based on environment
        const bootstrapNodes = getBootstrapNodes(
          import.meta.env.PROD ? 'production' : 'development'
        )

        if (bootstrapNodes.length > 0) {
          console.log('Initializing P2P with bootstrap nodes:', bootstrapNodes)
          
          // Set bootstrap nodes in the plugin
          await invoke('messaging_set_bootstrap_nodes', {
            bootstrapNodes,
          })
          
          console.log('P2P bootstrap nodes configured successfully')
        } else {
          console.warn('No bootstrap nodes configured')
        }
      } catch (error) {
        console.error('Failed to initialize P2P configuration:', error)
      }
    }

    initP2P()
  }, []) // Empty deps = run once on mount
}
