import {
  formatShortPeerId,
  formatPeerIdForDisplay,
  formatShortGroupId,
} from '../peerUtils'

describe('peerUtils', () => {
  describe('formatShortPeerId', () => {
    it('should return original ID if length is 12 or less', () => {
      expect(formatShortPeerId('12ab34')).toBe('12ab34')
      expect(formatShortPeerId('12ab34cd')).toBe('12ab34cd')
      expect(formatShortPeerId('12ab34cd56ef')).toBe('12ab34cd56ef')
    })

    it('should shorten long IDs by keeping first 6 and last 6 characters', () => {
      const longId = '12ab34cd56ef78gh90ij34kl56mn78op90qr'
      const expected = '12ab34...op90qr'
      expect(formatShortPeerId(longId)).toBe(expected)
    })

    it('should handle empty string', () => {
      expect(formatShortPeerId('')).toBe('')
    })

    it('should handle exactly 13 characters', () => {
      const id = '12ab34cd56ef7'
      const expected = '12ab34...ef7'
      expect(formatShortPeerId(id)).toBe(expected)
    })

    it('should work for group IDs too', () => {
      const groupId = 'group1234567890abcdef1234567890'
      const expected = 'group1...7890'
      expect(formatShortPeerId(groupId)).toBe(expected)
    })
  })

  describe('formatPeerIdForDisplay', () => {
    it('should use short format by default', () => {
      const longId = '12ab34cd56ef78gh90ij34kl56mn78op90qr'
      const expected = '12ab34...op90qr'
      expect(formatPeerIdForDisplay(longId)).toBe(expected)
    })

    it('should use full format when shortFormat is false', () => {
      const longId = '12ab34cd56ef78gh90ij34kl56mn78op90qr'
      expect(formatPeerIdForDisplay(longId, false)).toBe(longId)
    })

    it('should handle empty string', () => {
      expect(formatPeerIdForDisplay('')).toBe('')
    })
  })

  describe('formatShortGroupId', () => {
    it('should be an alias for formatShortPeerId', () => {
      const groupId = 'group1234567890abcdef1234567890'
      expect(formatShortGroupId(groupId)).toBe(formatShortPeerId(groupId))
    })
  })
})
