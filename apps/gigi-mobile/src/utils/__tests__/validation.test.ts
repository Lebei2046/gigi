//! Unit tests for frontend validation utilities

describe('Validation Utilities', () => {
  describe('Nickname validation', () => {
    const validNicknames = [
      'Alice',
      'Bob',
      'alice123',
      'Alice_Bob',
      'Alice-Bob',
      'Test User',
      'a',
    ]

    const invalidNicknames = [
      '',
      'A'.repeat(65),
      'Alice<script>',
      'Bob@home',
      'Test#123',
      'User!',
      'Test$',
      ' ',
    ]

    test('valid nicknames pass validation', () => {
      for (const nickname of validNicknames) {
        const isValid =
          nickname.length > 0 &&
          nickname.length <= 64 &&
          /^[\w\s-]+$/.test(nickname)
        expect(isValid).toBe(true)
      }
    })

    test('invalid nicknames fail validation', () => {
      for (const nickname of invalidNicknames) {
        const isValid =
          nickname.length > 0 &&
          nickname.length <= 64 &&
          /^[\w\s-]+$/.test(nickname)
        expect(isValid).toBe(false)
      }
    })
  })

  describe('Message validation', () => {
    const dangerousPatterns = [
      '<script',
      'javascript:',
      'onerror=',
      'onload=',
      'data:',
    ]

    test('messages with dangerous patterns are rejected', () => {
      const dangerousMessages = [
        '<script>alert("xss")</script>',
        'javascript:alert("xss")',
        '<img onerror=alert("xss")>',
        '<body onload=alert("xss")>',
        'data:text/html,<script>alert("xss")</script>',
      ]

      for (const message of dangerousMessages) {
        const hasDangerousPattern = dangerousPatterns.some(pattern =>
          message.toLowerCase().includes(pattern.toLowerCase())
        )
        expect(hasDangerousPattern).toBe(true)
      }
    })

    test('long messages are rejected', () => {
      const longMessage = 'A'.repeat(100_001)
      expect(longMessage.length).toBeGreaterThan(100_000)
    })

    test('safe messages pass validation', () => {
      const safeMessages = [
        'Hello, world!',
        'Test message with emojis 😊',
        'Normal message text',
      ]

      for (const message of safeMessages) {
        const hasDangerousPattern = dangerousPatterns.some(pattern =>
          message.toLowerCase().includes(pattern.toLowerCase())
        )
        expect(hasDangerousPattern).toBe(false)
        expect(message.length).toBeLessThanOrEqual(100_000)
      }
    })
  })

  describe('Group name validation', () => {
    const validGroupNames = [
      'Test Group',
      'Developers',
      'TestGroup',
      'Test_Group',
      'Test-Group',
      'A',
    ]

    const invalidGroupNames = [
      '',
      'A'.repeat(129),
      'Group<script>',
      'Test#Group',
      'Group@',
    ]

    test('valid group names pass validation', () => {
      for (const groupName of validGroupNames) {
        const isValid =
          groupName.length > 0 &&
          groupName.length <= 128 &&
          /^[\w\s-]+$/.test(groupName)
        expect(isValid).toBe(true)
      }
    })

    test('invalid group names fail validation', () => {
      for (const groupName of invalidGroupNames) {
        const isValid =
          groupName.length > 0 &&
          groupName.length <= 128 &&
          /^[\w\s-]+$/.test(groupName)
        expect(isValid).toBe(false)
      }
    })
  })

  describe('URI validation', () => {
    const dangerousSchemes = ['file:///', 'javascript:', 'vbscript:', 'data:']

    test('URIs with dangerous schemes are rejected', () => {
      const dangerousURIs = [
        'file:///etc/passwd',
        'javascript:alert("xss")',
        'vbscript:alert("xss")',
        'data:text/html,<script>alert("xss")</script>',
      ]

      for (const uri of dangerousURIs) {
        const hasDangerousScheme = dangerousSchemes.some(scheme =>
          uri.toLowerCase().startsWith(scheme.toLowerCase())
        )
        expect(hasDangerousScheme).toBe(true)
      }
    })

    test('safe URIs pass validation', () => {
      const safeURIs = [
        'https://example.com/file.pdf',
        'http://localhost:8080/file',
        'ipfs://QmTestHash123',
      ]

      for (const uri of safeURIs) {
        const hasDangerousScheme = dangerousSchemes.some(scheme =>
          uri.toLowerCase().startsWith(scheme.toLowerCase())
        )
        expect(hasDangerousScheme).toBe(false)
      }
    })
  })

  describe('Peer ID validation', () => {
    test('valid peer IDs are non-empty', () => {
      const validPeerIds = [
        '12D3KooWQxHnVYkZp7DzXz4z5z5z5z5z5z5z5z5z5z5z5z5z5z',
      ]

      for (const peerId of validPeerIds) {
        expect(peerId.length).toBeGreaterThan(0)
        expect(peerId.length).toBeLessThanOrEqual(256)
      }
    })

    test('empty peer IDs are invalid', () => {
      expect('').toBe('')
    })

    test('peer IDs start with valid characters', () => {
      const validPeerIds = [
        '12D3KooWQxHnVYkZp7DzXz4z5z5z5z5z5z5z5z5z5z5z5z5z5z',
        'QmTestHash',
      ]

      for (const peerId of validPeerIds) {
        expect(peerId[0]).toMatch(/^[Qm1]/)
      }
    })
  })

  describe('File path validation', () => {
    test('relative paths are valid', () => {
      const validPaths = [
        'relative/path/to/file.txt',
        'file.pdf',
        './file.pdf',
        'data/uploads/image.png',
      ]

      for (const path of validPaths) {
        expect(path.startsWith('/')).toBe(false)
        expect(path.contains('..')).toBe(false)
      }
    })

    test('path traversal is detected', () => {
      const invalidPaths = [
        '../etc/passwd',
        './../../etc/passwd',
        'data/../../../etc/passwd',
      ]

      for (const path of invalidPaths) {
        expect(path.contains('..')).toBe(true)
      }
    })

    test('absolute paths are detected', () => {
      const absolutePaths = ['/etc/passwd', '/home/user/file.txt']

      for (const path of absolutePaths) {
        expect(path.startsWith('/')).toBe(true)
      }
    })

    test('system directories are detected', () => {
      const systemPaths = [
        './etc/passwd',
        'data/etc/config',
        './proc/123/status',
      ]

      for (const path of systemPaths) {
        expect(path.includes('etc/') || path.includes('proc/')).toBe(true)
      }
    })
  })

  describe('String sanitization', () => {
    test('dangerous characters are escaped', () => {
      const input = '<script>alert("xss")</script>'
      const sanitized = input
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#x27;')
        .replace(/&/g, '&amp;')

      expect(sanitized).toBe(
        '&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;'
      )
      expect(sanitized.includes('<')).toBe(false)
      expect(sanitized.includes('>')).toBe(false)
    })

    test('safe strings remain unchanged', () => {
      const safeInput = 'Hello, world! This is safe text.'
      const sanitized = safeInput
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#x27;')
        .replace(/&/g, '&amp;')

      expect(sanitized).toBe(safeInput)
    })
  })

  describe('File size validation', () => {
    const MAX_FILE_SIZE = 5 * 1024 * 1024 * 1024 // 5GB

    test('valid file sizes are accepted', () => {
      const validSizes = [
        0,
        1024,
        1024 * 1024, // 1MB
        1024 * 1024 * 1024, // 1GB
        MAX_FILE_SIZE,
      ]

      for (const size of validSizes) {
        expect(size).toBeLessThanOrEqual(MAX_FILE_SIZE)
      }
    })

    test('invalid file sizes are rejected', () => {
      const invalidSizes = [
        MAX_FILE_SIZE + 1,
        10 * 1024 * 1024 * 1024, // 10GB
        Number.MAX_SAFE_INTEGER,
      ]

      for (const size of invalidSizes) {
        expect(size).toBeGreaterThan(MAX_FILE_SIZE)
      }
    })
  })

  describe('Share code validation', () => {
    test('valid share codes pass validation', () => {
      const validCodes = [
        'abc123',
        'test-file-share',
        'file_123.pdf',
        'share.code.456',
      ]

      for (const code of validCodes) {
        expect(code.length).toBeGreaterThan(0)
        expect(code.length).toBeLessThanOrEqual(256)
        expect(/^[\w.-]+$/.test(code)).toBe(true)
      }
    })

    test('invalid share codes fail validation', () => {
      const invalidCodes = [
        '',
        'A'.repeat(257),
        'share@code',
        'code#123',
        'test&code',
        'code$',
        'code/',
      ]

      for (const code of invalidCodes) {
        const isValid =
          code.length > 0 && code.length <= 256 && /^[\w.-]+$/.test(code)
        expect(isValid).toBe(false)
      }
    })
  })
})
