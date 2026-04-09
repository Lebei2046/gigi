# Gigi P2P Ecosystem Guidelines

This document serves as a comprehensive guideline for the Gigi P2P ecosystem, describing its components, architecture, and usage. It provides an overview of the entire ecosystem and references detailed documentation for each component.

## 1. Components Overview

### Ecosystem Overview

The Gigi P2P ecosystem is a decentralized network of components designed to enable secure, direct communication between peers without relying on centralized servers. It is built on top of Libp2p, a modular network stack for peer-to-peer applications.

### Projects Index

#### TypeScript Projects
- **@gigi/amp**: AMP (Agent Messaging Protocol) implementation
- **@gigi/logging**: Logging utilities using Pino
- **@gigi/mdns**: mDNS functionality for local peer discovery
- **@gigi/p2p**: High-level TypeScript client for P2P communication
- **@gigi/@request-response**: Request-response protocol implementation
- **p2p-example**: Example usage of the Gigi P2P client

#### Rust Projects
- **gigi-auth**: Authentication and key management
- **gigi-dns**: Decentralized name resolution
- **gigi-file-sharing**: File sharing utilities
- **gigi-p2p**: Core P2P functionality with high performance
- **gigi-store**: Persistence layer for data storage
- **tauri-plugin-gigi**: Integration with Tauri desktop/mobile apps

#### Applications
- **gigi-mobile**: Mobile application for Gigi P2P network
- **gigi-node**: Standalone network node for bootstrap/relay
- **gigi-openclaw**: Integration plugin for OpenClaw chat application

## 2. Project Structure Rules

### TypeScript Project Rules

1. **Project Structure**
   - Source code in `src/` directory
   - Tests in `src/__tests__/` directory
   - Configuration files at root:
     - `package.json` for dependencies and scripts
     - `tsconfig.json` for TypeScript configuration
     - `eslint.config.js` for ESLint configuration
     - `.prettierrc` for Prettier configuration
     - `README.md` for documentation

2. **Testing**
   - Use Vitest for testing with built-in TypeScript support
   - Test files follow `*.test.ts` naming convention
   - Place tests in `__tests__` directories
   - Aim for 80%+ test coverage

3. **Linting and Formatting**
   - Use ESLint for code linting
   - Use Prettier for code formatting
   - Include `format-all` script that runs `prettier --write . && eslint "src/**/*.{ts,tsx,js,jsx}" --ext ts,tsx,js,jsx --fix`

4. **Package Management**
   - Use pnpm as the package manager
   - Maintain consistent dependency patterns across projects

5. **Logging**
   - Use Pino for structured JSON logging
   - Centralize logging utilities in the `@gigi/logging` package
   - Replace `console.log` with structured logging

### Rust Project Rules

1. **Project Structure**
   - Source code in `src/` directory
   - Tests in `tests/` directory
   - Configuration files at root:
     - `Cargo.toml` for dependencies and configuration
     - `README.md` for documentation
   - Optional directories:
     - `examples/` for example code
     - `docs/` for documentation

2. **Testing**
   - Use Rust's built-in test framework
   - Test files follow `*_test.rs` naming convention
   - Place tests in the `tests/` directory
   - Include both unit and integration tests

3. **Code Organization**
   - Use modules with `mod.rs` files
   - Follow Rust's standard project structure
   - Maintain clear separation of concerns

4. **Package Management**
   - Use Cargo as the package manager
   - Maintain consistent dependency patterns across projects

5. **Documentation**
   - Include comprehensive documentation in `docs/` directory
   - Use Rustdoc for code documentation

## 3. Architecture

### 3.1 Layers

The Gigi P2P ecosystem follows a layered architecture:

1. **Network Layer**: Provides the foundation for peer-to-peer communication
2. **Protocol Layer**: Defines the rules for communication between peers
3. **Application Layer**: Provides higher-level functionality for end users

For detailed information about the architecture, see [docs/architecture.md](docs/architecture.md).

### 3.2 Network Topology

The Gigi P2P network uses a combination of several peer discovery and routing mechanisms:

- **Kademlia DHT**: Distributed hash table for peer discovery and content routing
- **mDNS**: Local peer discovery on the same network
- **GossipSub**: Topic-based publish/subscribe messaging for groups
- **Circuit Relay**: NAT traversal for peers behind firewalls

### 3.3 Technology Stacks

- **Libp2p**: Core P2P networking library
- **TypeScript**: For TypeScript client and OpenClaw plugin
- **Rust**: For Rust client and network node
- **Tauri**: For desktop and mobile integration
- **SQLite**: For data persistence in Gigi Store

### 3.4 Scalability Considerations

The Gigi P2P ecosystem is designed to scale efficiently:

- **Horizontal Scaling**: Decentralized design with no single point of failure
- **Performance Optimization**: Connection pooling, parallel processing, and caching
- **Network Resilience**: Redundancy, fault tolerance, and self-healing

### 3.5 Security Architecture

The Gigi P2P ecosystem implements multiple layers of security:

- **Transport Security**: Noise protocol for encrypted communication
- **Application Security**: Access control, message verification, and rate limiting
- **Data Security**: Encryption at rest, key management, and data integrity

For detailed information about the security architecture, see [docs/architecture.md](docs/architecture.md).

### 3.6 Security Considerations

- **Encryption**: All communications are encrypted
- **Peer Verification**: Peers are verified by their public keys
- **Access Control**: Configure who can send you messages
- **File Safety**: Be cautious when downloading files from unknown peers
- **Privacy**: Protect user privacy through anonymous communication

## 4. Best Practices

### 4.1 Testing Best Practices for Node.js/TypeScript

Testing is crucial for maintaining code quality, catching bugs early, and ensuring your application works as expected. Follow these guidelines for testing Node.js/TypeScript projects in the Gigi P2P ecosystem:

#### Key Testing Principles

- **Framework Selection**: Use Vitest for faster testing with built-in TypeScript support, or Jest for broader compatibility.
- **Test Organization**: Place tests in `__tests__` directories or use `.test.ts` naming convention.
- **Type Safety**: Leverage TypeScript's type system in tests to catch errors early.
- **Mocking**: Properly mock external dependencies like libp2p to isolate tests.
- **Coverage Targets**: Aim for 80%+ test coverage, focusing on critical paths.
- **CI/CD Integration**: Automate tests in GitHub Actions for every push and pull request.
- **Test Quality**: Write descriptive, isolated tests that cover edge cases and error scenarios.
- **P2P-Specific Testing**: Mock libp2p, test network simulations, and verify file sharing functionality.
- **Continuous Improvement**: Regularly review coverage reports and update tests as code changes.

For detailed documentation and examples, see [docs/testing-best-practices.md](docs/testing-best-practices.md).

### 4.2 Best Practices for Rust Projects

- **Code Quality**: Use rustfmt for code formatting and clippy for linting
- **Error Handling**: Use Result type and proper error propagation
- **Testing**: Write comprehensive unit and integration tests
- **Documentation**: Document public APIs with Rustdoc
- **Performance**: Profile and optimize critical paths
- **Safety**: Leverage Rust's type system and ownership model for memory safety

## 5. Contribution & Support

### 5.1 Contributing

Contributions to the Gigi P2P ecosystem are welcome! Please see the project's GitHub repository for guidelines.

### 5.2 Troubleshooting

#### Common Issues
- **Connection Problems**: Check network connectivity and firewall settings
- **Peer Discovery**: Ensure mDNS and DHT are enabled
- **File Transfer**: Check file permissions and network stability
- **Group Messaging**: Ensure all peers are subscribed to the same topic
- **NAT Traversal**: Ensure relay nodes are available

#### Logs
Check the application logs and Gigi client logs for detailed error information.

### 5.3 Getting Started

For a step-by-step guide to getting started with the Gigi P2P ecosystem, see [docs/quick-start.md](docs/quick-start.md).

## 6. Additional Resources

### API Reference

For detailed API documentation for all components, see [docs/api-reference.md](docs/api-reference.md).

### Examples

For practical examples of how to use the Gigi P2P ecosystem, see [docs/examples.md](docs/examples.md).