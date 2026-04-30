# Testing Best Practices for Node.js/TypeScript

Testing is crucial for maintaining code quality, catching bugs early, and ensuring your application works as expected. This document provides comprehensive guidelines for testing Node.js/TypeScript projects in the Gigi P2P ecosystem.

## 1. Choose the Right Testing Framework

### Popular Frameworks
- **Jest**: Widely used, built-in mocking, snapshot testing, and coverage reporting.
- **Vitest**: Faster alternative to Jest, ESM-first, built-in TypeScript support.
- **Mocha + Chai**: Flexible, customizable, good for complex test scenarios.

### Setup Example (Vitest)
```bash
# Install dependencies
pnpm add -D vitest

# Add test scripts to package.json
{
  "scripts": {
    "test": "vitest",
    "test:coverage": "vitest --coverage"
  }
}
```

## 2. Organize Tests Effectively

### File Structure
- **Option 1**: Place tests in a `__tests__` directory alongside source files.
- **Option 2**: Name test files with `.test.ts` extension next to source files.

### Example Structure
```
src/
  client.ts
  client.test.ts
  __tests__/
    integration.test.ts
```

## 3. TypeScript-Specific Practices

### Type Safety in Tests
- Use TypeScript types in test assertions to catch type errors.
- Leverage `as const` for literal types in test data.
- Enable `strict` mode in `tsconfig.json` for tests.

### tsconfig.json for Tests
```json
{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "noEmit": true,
    "types": ["vitest", "node"]
  },
  "include": ["**/*.test.ts"]
}
```

## 4. Mocking & Stubbing

### Mocking Dependencies
- Use framework-specific mocking for external dependencies.
- Mock entire modules or specific functions.

### Example Mock
```typescript
// Mock libp2p instance
const mockLibp2p = {
  handle: vi.fn(),
  dialProtocol: vi.fn().mockResolvedValue({
    sink: vi.fn(),
    source: {
      [Symbol.asyncIterator]: async function* () {
        yield new TextEncoder().encode(JSON.stringify({ type: 'pong' }));
      }
    },
    close: vi.fn()
  })
};
```

## 5. Test Coverage

### Tools
- **Vitest/Jest**: Built-in coverage reporting.
- **Istanbul**: Standalone coverage tool.

### Coverage Targets
- Aim for 80%+ coverage, focusing on critical paths.
- Exclude generated files, third-party code, and configuration.

### Coverage Configuration
```json
// vitest.config.ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  coverage: {
    reporter: ['text', 'json', 'html'],
    thresholds: {
      global: {
        branches: 80,
        functions: 80,
        lines: 80,
        statements: 80
      }
    }
  }
});
```

## 6. CI/CD Integration

### GitHub Actions Example
```yaml
# .github/workflows/test.yml
name: Test
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: pnpm/action-setup@v2
        with:
          version: latest
      - uses: actions/setup-node@v3
        with:
          node-version: 18
          cache: 'pnpm'
      - run: pnpm install
      - run: pnpm test
      - run: pnpm test --coverage
```

## 7. Best Practices for Writing Tests

### Test Writing
- **Descriptive Names**: Use clear, concise test names (e.g., `should return 404 when user not found`).
- **Edge Cases**: Test empty inputs, invalid data, and error scenarios.
- **Setup/Teardown**: Use `beforeAll`, `afterAll`, `beforeEach`, `afterEach` for shared setup.
- **Isolation**: Tests should be independent and not rely on each other.
- **Avoid Flaky Tests**: Use fixed data, avoid timing-dependent tests.

### Example Test
```typescript
describe('P2pClient', () => {
  let client: P2pClient;

  beforeEach(() => {
    client = new P2pClient({
      nickname: 'Test Client',
      config: {
        bootstrapNodes: [],
        enableKademlia: false,
        enableRelay: true,
        enableMdns: false,
        listenAddrs: ['/ip4/0.0.0.0/tcp/0']
      }
    });
  });

  afterEach(async () => {
    if (client) await client.stop();
  });

  test('should start successfully', async () => {
    await client.start();
    expect(client.isStarted()).toBe(true);
  });

  test('should share file and return share code', async () => {
    await client.start();
    const shareCode = await client.shareFile('./README.md');
    expect(typeof shareCode).toBe('string');
    expect(shareCode.length).toBeGreaterThan(0);
  });
});
```

## 8. P2P-Specific Testing Scenarios

For P2P projects like Gigi P2P:
- **Mock libp2p**: Simulate peer discovery, message delivery, and file transfer.
- **Integration Tests**: Test communication between multiple client instances.
- **Network Simulation**: Test scenarios with NAT traversal, relay usage, and offline peers.
- **File Sharing**: Test file chunking, progress tracking, and error handling.
- **Group Messaging**: Test GossipSub functionality with multiple peers.

## 9. Continuous Improvement

- **Refactor Tests**: Keep tests clean and maintainable.
- **Review Test Coverage**: Regularly check coverage reports to identify untested code.
- **Update Tests**: Keep tests in sync with code changes.
- **Test Automation**: Integrate tests into your development workflow.

By following these practices, you'll build a robust test suite that ensures your Node.js/TypeScript projects in the Gigi P2P ecosystem are reliable and maintainable.
