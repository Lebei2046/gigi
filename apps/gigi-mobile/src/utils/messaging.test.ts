import { getPeerId } from '@/utils/messaging'
import { describe, expect, it } from 'vitest'

describe('messaging functions', () => {
  describe('getPeerId', () => {
    it('should generate a valid peer id', async () => {
      const privKey = new Uint8Array([
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
      ])
      const peerId = await getPeerId(privKey)
      expect(peerId).toEqual(
        '12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X'
      )
    })
  })
})
