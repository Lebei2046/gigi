# Gigi Dioxus Frontend

This is the Dioxus-based frontend for the Gigi P2P ecosystem, providing a user-friendly interface for account creation and management.

## Project Overview

The Gigi Dioxus frontend is built using [Dioxus](https://dioxuslabs.com/), a Rust framework for building user interfaces. It implements the signup functionality for the Gigi P2P network, allowing users to create new accounts or import existing ones using seed phrases.

## Features

- **Account Creation**: Create a new account with a generated seed phrase
- **Account Import**: Import an existing account using a seed phrase
- **Seed Phrase Verification**: Confirm seed phrases to ensure they're saved correctly
- **Account Information**: Set up account details like name and password
- **Group Creation**: Option to create the first chat group during signup

## Getting Started

### Prerequisites

- Rust and Cargo (https://www.rust-lang.org/tools/install)
- Dioxus CLI (`cargo install dioxus-cli`)

### Running the App

Run the following command in the `apps/gigi-dioxus` directory to start the development server:

```bash
dx serve
```

This will start a web server at `http://localhost:8080` where you can access the application.

### Building for Production

To build the application for production, run:

```bash
dx build --release
```

### Platform Support

The app can be built for different platforms:

```bash
# Web (default)
dx serve

# Desktop
dx serve --platform desktop

# Mobile
dx serve --platform mobile
```

## Dependencies

- **Dioxus 0.7.4**: UI framework for Rust
- **Tailwind CSS**: Styling framework
- **Futures**: For async operations

## Notes

### Futures Compatibility

This project uses Dioxus 0.7.4, which requires a patch to work with newer versions of the `futures` library. The patches are located in the `patches/` directory at the project root.

### P2P Integration

The frontend is designed to work with the Gigi P2P backend, which provides secure, decentralized communication between peers.

### Testing with Multiple Instances

Before testing, ensure the project compiles successfully:

```bash
# Check compilation
dx build --desktop
```

To test P2P communication between different accounts, you can run multiple instances of the app with distinct data directories:

```bash
# Run first instance (e.g., Alice)
GIGI_DATA_DIR=~/.gigi-dioxus-alice dx serve --desktop

# Run second instance (e.g., Bob)
GIGI_DATA_DIR=~/.gigi-dioxus-bob dx serve --desktop
```

Each instance will use its own data directory, ensuring they don't interfere with each other. This allows you to test:

- Direct messaging between different accounts
- Group chat functionality
- File sharing between peers
- Peer discovery and connection

### Default Data Storage Location

If `GIGI_DATA_DIR` is not set, the app will use the default platform-specific data directory:

- **Linux**: `~/.local/share/gigi-dioxus/`
- **macOS**: `~/Library/Application Support/gigi-dioxus/`
- **Windows**: `C:\Users\{username}\AppData\Local\gigi-dioxus\`

## Contributing

Contributions to the Gigi Dioxus frontend are welcome! Please follow the project's coding conventions and submit pull requests for any improvements or bug fixes.

