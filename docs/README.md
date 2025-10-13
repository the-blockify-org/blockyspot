# Blockyspot Documentation

Welcome to the Blockyspot documentation! This directory contains comprehensive guides and references for understanding and using Blockyspot.

## Documentation Files

### [Get Started](get-started.md)
Start here to learn what Blockyspot is, why it was created, and how to navigate the documentation.

### [Protocol & Format](format.md)
Details on the communication protocol:
- Command format and available commands
- Socket communication and connection flow
- Event format and types
- Command management architecture

### [Server Architecture](server.md)
How the WebSocket server works:
- Server implementation details
- Connection handling
- Current limitations
- Configuration options

### [Librespot Integration](librespot.md)
How Librespot is integrated:
- Why Librespot is used
- What components we use
- What Blockyspot adds on top
- Initialization flow and configuration

### [The Audio Sink](sink.md)
Deep dive into audio streaming:
- Custom WebSocketSink implementation
- Audio pipeline and data flow
- Buffering and timing
- Design decisions

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
