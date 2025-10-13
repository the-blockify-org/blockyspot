# Get Started

Welcome to the Blockyspot documentation! This guide will help you understand what Blockyspot is and how to navigate the documentation.

## What is Blockyspot?

Blockyspot is a service built on top of the [Librespot](https://github.com/librespot-org/librespot) crate that provides a WebSocket-based API for applications in any language to easily interact with Spotify Connect. The project was primarily created for a Minecraft mod called "Blockify" that allows players to connect their Spotify accounts and stream music with friends.

## Why Blockyspot?

The [Librespot-java](https://github.com/librespot-org/librespot-java) port is now deprecated and may soon stop working. While the main Rust implementation is stable, the standard Librespot binaries aren't designed to process input commands like play, pause, volume control, shuffle, and all the other Spotify Connect commands that applications need. Blockyspot bridges this gap by providing a service layer that:

- Embeds the Librespot library as a controllable service
- Exposes a WebSocket API for real-time bidirectional communication
- Supports multiple concurrent Spotify Connect sessions
- Enables applications to act as full-featured Spotify Connect clients

## What Can You Do with Blockyspot?

With Blockyspot, you can:

- **Create virtual Spotify Connect devices** that appear in your Spotify app
- **Control playback** using commands over WebSockets (play, pause, skip, volume, etc.)
- **Stream audio data** to your application in real-time
- **Manage multiple devices** simultaneously from a single server instance
- **Receive events** about playback state changes and audio stream status

## Current State

Blockyspot is in **alpha stage**. The project currently:

- Supports basic playback commands (play, pause, next, previous, volume control, etc.)
- Uses WebSockets for communication (this may change in future versions)
- Allows multiple Spotify Connect sessions per server instance
- Has an unstandardized event format that is still being refined
- Lacks authentication and has messy session management

The project is being documented now to preserve knowledge of its current architecture and functionality for future development.

## Documentation Structure

The documentation is organized into focused topics. Choose what you need to read:

### 📋 [Protocol & Format](format.md)
Learn about the communication protocol:
- Command format and available commands
- WebSocket connection and response format
- Event types and their structure
- Command management architecture

### 🖥️ [Server Architecture](server.md)
Understand how the server works:
- WebSocket server implementation
- Connection and state management
- Current limitations
- Configuration options

### 🎵 [Librespot Integration](librespot.md)
Deep dive into Librespot usage:
- Why Librespot is used
- What components we use
- How initialization works
- Configuration details

### 🔊 [The Audio Sink](sink.md)
How audio streaming works:
- Custom WebSocketSink implementation
- Audio pipeline and data flow
- Buffering and timing
- Design decisions

## Tips for Reading the Docs

> [!TIP]
> **For Quick Reference**: Each documentation file includes code references with links to the actual source files. Click these to navigate directly to the implementation.

> [!TIP]
> **For Understanding the Flow**: Start with [Protocol & Format](format.md) to understand how to communicate with Blockyspot, then read [Server Architecture](server.md) to see how it's implemented.

> [!TIP]
> **For Audio Streaming**: Read [Librespot Integration](librespot.md) first to understand the foundation, then [The Audio Sink](sink.md) to see how we stream audio over WebSockets.

> [!NOTE]
> The documentation reflects the current alpha state of the project. Some aspects (like event format and session management) are still being refined and may change in future versions.
