# @gigi/logging

A structured logging utility for the Gigi P2P ecosystem, built on top of Pino.

## Overview

`@gigi/logging` provides a standardized logging solution for all components in the Gigi P2P ecosystem. It offers structured JSON logging with sensible defaults and custom serializers for P2P-specific data types.

## Features

- **Structured JSON Logging**: Uses Pino for high-performance, structured JSON logging
- **P2P-Specific Serializers**: Built-in serializers for peer IDs and multiaddresses
- **Environment-Aware Leveling**: Automatically adjusts log level based on environment
- **Customizable**: Supports custom log levels, names, and serializers

## Installation

```bash
pnpm add @gigi/logging
```

## Usage

### Basic Usage

```typescript
import { createLogger } from '@gigi/logging';

const logger = createLogger({ name: 'my-component' });

logger.info('Component started');
logger.debug('Detailed information');
logger.error('An error occurred', { error: new Error('Something went wrong') });
```

### With Custom Options

```typescript
import { createLogger } from '@gigi/logging';

const logger = createLogger({
  name: 'my-service',
  level: 'warn', // Override default level
  serializers: {
    // Custom serializer for additional data types
    customData: (data) => {
      return data ? data.toString() : null;
    }
  }
});

logger.warn('Warning message', { customData: { id: 123, name: 'test' } });
```

### P2P-Specific Serialization

```typescript
import { createLogger } from '@gigi/logging';

const logger = createLogger({ name: 'p2p-client' });

// Peer ID and multiaddress will be automatically serialized
logger.info('Connected to peer', {
  peerId: somePeerId, // Will be converted to string
  multiaddr: someMultiaddr // Will be converted to string
});
```

## API

### `createLogger(options?: LoggerOptions): Logger`

Creates a new Pino logger instance with Gigi-specific defaults.

#### Parameters

- `options` (optional): Configuration options
  - `level` (string): Log level (default: 'debug' in development, 'info' in production)
  - `name` (string): Logger name (default: 'gigi')
  - `serializers` (Record<string, pino.SerializerFn>): Custom serializers

#### Returns

- A Pino logger instance

## Default Serializers

- **peerId**: Converts peer ID objects to strings
- **multiaddr**: Converts multiaddress objects to strings
- **error**: Uses Pino's standard error serializer

## Environment Variables

- `LOG_LEVEL`: Overrides the default log level
- `NODE_ENV`: Determines default log level ('production' uses 'info', others use 'debug')

## Dependencies

- [pino](https://github.com/pinojs/pino): High-performance logging library

## Testing

```bash
# Run tests
pnpm test

# Run tests with coverage
pnpm test:coverage
```

## Linting and Formatting

```bash
# Lint code
pnpm lint

# Fix linting issues
pnpm lint:fix

# Format code
pnpm format
```
