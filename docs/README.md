# Blockyspot Documentation

Welcome to the Blockyspot documentation! This directory contains comprehensive guides and references for understanding and using Blockyspot.

## Documentation Files

### [Get Started](get-started.md)

The main documentation file that covers:

- **Introduction** - What Blockyspot is, why it was created, and its current state
- **Format** - Details on the communication protocol:
  - Command format and available commands
  - Socket communication and connection flow
  - Event format and types
  - Command management architecture
- **Server** - How the WebSocket server works, its architecture, and current limitations
- **Librespot** - How Librespot is integrated and what Blockyspot adds on top
- **The Sink** - Deep dive into the custom WebSocketSink implementation

## Quick Links

- [Main README](../README.md) - Project overview and quick start
- [Source Code](../src/) - Blockyspot source code
- [Commands](../src/commands.rs) - Command definitions
- [Server](../src/server.rs) - WebSocket server implementation
- [Spotify Client](../src/spotify.rs) - Librespot integration
- [WebSocket Sink](../src/ws_sink.rs) - Audio streaming over WebSockets

## Project Status

Blockyspot is currently in **alpha stage**. The documentation reflects the current implementation and will be updated as the project evolves.

## Contributing

When contributing to Blockyspot, please update the relevant documentation to reflect your changes. The documentation uses Markdown format with GitHub-style alerts for important information.
