# Librespot Integration

[Librespot](https://github.com/librespot-org/librespot) is an open-source Spotify Connect client library written in Rust. It's the foundation that Blockyspot builds upon.

## Table of Contents
- [Why Librespot?](#why-librespot)
- [What We Use from Librespot](#what-we-use-from-librespot)
- [What We Build On Top](#what-we-build-on-top)
- [Initialization Flow](#initialization-flow)
- [Librespot Configuration](#librespot-configuration)

## Why Librespot?

Librespot provides:
- **Full Spotify Connect protocol implementation** - Allows devices to appear in Spotify's device list
- **Audio streaming** - Decodes and streams audio from Spotify's servers
- **Playback control** - Handles all Spotify Connect commands (play, pause, seek, etc.)
- **Authentication** - Manages OAuth tokens and session credentials
- **Caching** - Stores credentials and audio data for offline playback

## What We Use from Librespot

Blockyspot uses several Librespot crates (all from the `dev` branch):

1. **`librespot-core`** ([`src/spotify.rs`](../src/spotify.rs))
   - `Session` - Manages the Spotify session
   - `Credentials` - Handles authentication with access tokens
   - `Cache` - Stores session data and credentials
   - `SessionConfig` - Configures the session

2. **`librespot-connect`** ([`src/spotify.rs`](../src/spotify.rs))
   - `Spirc` - Implements the Spotify Connect protocol
   - `ConnectConfig` - Configures the Connect device (name, device type, etc.)

3. **`librespot-playback`** ([`src/spotify.rs`](../src/spotify.rs), [`src/ws_sink.rs`](../src/ws_sink.rs))
   - `Player` - Manages audio playback
   - `PlayerConfig` - Configures the player
   - `PlayerEvent` - Notifies about playback state changes
   - `AudioFormat` - Defines audio format specifications
   - `Sink` - Audio output backend trait
   - `Converter` - Converts between audio formats
   - `mixer` - Volume control

## What We Build On Top

Librespot provides the protocol and playback, but we add:

1. **Remote Control API** - WebSocket interface for external applications
2. **Custom Audio Sink** - Stream audio data over WebSockets (see [The Audio Sink](sink.md))
3. **Multi-Device Support** - Multiple Spotify sessions per server instance
4. **Event Broadcasting** - Real-time notifications to connected clients
5. **Command Abstraction** - Higher-level API over raw Spirc calls

## Initialization Flow

When a `CreateDevice` command is received ([`src/spotify.rs`](../src/spotify.rs) lines 59-197):

1. **Session Creation** - A Librespot `Session` is created with cache and config
2. **Credentials** - Access token is used to create `Credentials`
3. **Audio Pipeline** - `Player` is created with our custom `WebSocketSink`
4. **Event Handlers** - Callbacks are registered for player and sink events
5. **Spirc Setup** - The Connect protocol handler is initialized and started
6. **Event Tasks** - Background tasks are spawned to forward events to WebSocket clients

## Librespot Configuration

Blockyspot uses mostly default configurations:
- **Cache location**: `.cache` directory (files in `.cache/files`)
- **Audio format**: Default (typically S16 stereo at 44.1kHz)
- **Mixer**: Default software mixer
- **Connect config**: Custom device name, default device type

> [!TIP]
> - Librespot integration: [`src/spotify.rs`](../src/spotify.rs)
> - The `SpotifyClient` struct wraps Librespot components
> - The `initialize()` method sets up the complete Librespot pipeline
> - Direct Spirc methods are wrapped via the `spirc_call!` macro (lines 25-40)
> - Librespot runs on the `dev` branch as specified in [`Cargo.toml`](../Cargo.toml)
