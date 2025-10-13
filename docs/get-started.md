# Get Started

## Table of Contents
- [Introduction](#introduction)
- [Format](#format)
  - [Command Format](#command-format)
  - [Socket Communication](#socket-communication)
  - [Event Format](#event-format)
- [Server](#server)
- [Librespot](#librespot)
- [The Sink](#the-sink)

## Introduction

Blockyspot is a service built on top of the [Librespot](https://github.com/librespot-org/librespot) crate that provides a WebSocket-based API for applications in any language to easily interact with Spotify Connect. The project was primarily created for a Minecraft mod called "Blockify" that allows players to connect their Spotify accounts and stream music with friends.

### Why Blockyspot?

The [Librespot-java](https://github.com/librespot-org/librespot-java) port is now deprecated and may soon stop working. While the main Rust implementation is stable, the standard Librespot binaries aren't designed to process input commands like play, pause, volume control, shuffle, and all the other Spotify Connect commands that applications need. Blockyspot bridges this gap by providing a service layer that:

- Embeds the Librespot library as a controllable service
- Exposes a WebSocket API for real-time bidirectional communication
- Supports multiple concurrent Spotify Connect sessions
- Enables applications to act as full-featured Spotify Connect clients

### Current State

Blockyspot is in **alpha stage**. The project currently:

- Supports basic playback commands (play, pause, next, previous, volume control, etc.)
- Uses WebSockets for communication (this may change in future versions)
- Allows multiple Spotify Connect sessions per server instance
- Has an unstandardized event format that is still being refined
- Lacks authentication and has messy session management

The project is being documented now to preserve knowledge of its current architecture and functionality for future development.

## Format

Blockyspot uses JSON-based messaging over WebSockets to communicate with clients. This section describes the various message formats used in the protocol.

### Command Format

Commands sent to Blockyspot follow a standardized JSON structure defined in [`src/commands.rs`](../src/commands.rs).

#### Command Message Structure

All commands use the `CommandMessage` structure:

```json
{
  "device_id": "optional-device-uuid",
  "command_type": "CommandName",
  "params": {
    "key": "value"
  }
}
```

- **`device_id`**: (Optional for `CreateDevice`, required for all other commands) The UUID of the device to control
- **`command_type`**: The name of the command to execute
- **`params`**: A JSON object containing command-specific parameters

#### Available Commands

The following commands are defined as the `Command` enum:

1. **CreateDevice** - Initialize a new Spotify Connect device
   ```json
   {
     "command_type": "CreateDevice",
     "params": {
       "token": "spotify_access_token",
       "device_name": "My Blockyspot Device"
     }
   }
   ```

2. **Playback Control Commands** (no parameters):
   - `Play` - Start playback
   - `PlayPause` - Toggle play/pause
   - `Pause` - Pause playback
   - `Prev` - Previous track
   - `Next` - Next track
   - `VolumeUp` - Increase volume
   - `VolumeDown` - Decrease volume
   - `Activate` - Activate the device
   - `Shutdown` - Shutdown the device

3. **State Control Commands** (boolean parameter):
   - `Shuffle` - Enable/disable shuffle
   - `Repeat` - Enable/disable repeat
   - `RepeatTrack` - Enable/disable repeat track

   Example:
   ```json
   {
     "device_id": "device-uuid",
     "command_type": "Shuffle",
     "params": {
       "state": true
     }
   }
   ```

4. **Position/Volume Commands**:
   - `SetPosition` - Set playback position in milliseconds
   - `SetVolume` - Set volume (0-65535)
   
   Example:
   ```json
   {
     "device_id": "device-uuid",
     "command_type": "SetVolume",
     "params": {
       "volume": 32768
     }
   }
   ```

5. **Disconnect** - Disconnect from Spotify
   ```json
   {
     "device_id": "device-uuid",
     "command_type": "Disconnect",
     "params": {
       "pause": true
     }
   }
   ```

> [!TIP]
> The command definitions and parsing logic are in [`src/commands.rs`](../src/commands.rs). The `Command::from_message()` method handles conversion from the wire format to internal command types.

### Socket Communication

The WebSocket server is implemented in [`src/server.rs`](../src/server.rs) and handles all client connections and message routing.

#### Connection Flow

1. Client connects to `ws://localhost:8888/ws` (port configurable via CLI)
2. Server sends a connection response:
   ```json
   {
     "status": "Connected to server",
     "protocol_version": "0.1.1"
   }
   ```
3. Client can now send commands

#### Response Format

All command responses follow a standard format defined by the `CommandResponse` structure in [`src/commands.rs`](../src/commands.rs):

```json
{
  "success": true,
  "message": "Response message",
  "data": {}
}
```

- **`success`**: Boolean indicating if the command succeeded
- **`message`**: Human-readable status message
- **`data`**: Optional additional data (e.g., device_id for CreateDevice)

#### Session Management

The server maintains a `ConnectionState` per WebSocket connection, which tracks multiple devices (Spotify sessions) using a `HashMap<String, SpotifyClient>`. Each device is identified by a UUID generated when created.

> [!TIP]
> - Server implementation: [`src/server.rs`](../src/server.rs)
> - The `SpotifyServer::process_ws_message()` method handles all incoming messages
> - The `SpotifyServer::handle_client_connection()` method manages the WebSocket lifecycle
> - Protocol version is defined as `PROTOCOL_VERSION` constant

### Event Format

Events are sent from the server to the client to notify about changes in playback state, audio stream status, and other important occurrences.

#### Event Types

Events are JSON messages with a `type` field that identifies the event category:

1. **Player Events** - Sent when playback state changes

   Defined by Librespot's `PlayerEvent` enum and handled in [`src/spotify.rs`](../src/spotify.rs):

   ```json
   {
     "type": "player_event",
     "device_id": "device-uuid",
     "data": {
       "event_type": "Playing",
       "details": {
         "play_request_id": 123,
         "track_id": "spotify:track:...",
         "position_ms": 1000
       }
     }
   }
   ```

   Possible event types:
   - `Playing` - Track started playing
   - `Paused` - Playback paused
   - `Stopped` - Playback stopped
   - `Loading` - Track is loading
   - `EndOfTrack` - Track finished

2. **Sink Events** - Sent when the audio sink status changes

   Generated in [`src/spotify.rs`](../src/spotify.rs) when the audio backend reports status:

   ```json
   {
     "type": "sink_event",
     "device_id": "device-uuid",
     "data": {
       "status": "Running"
     }
   }
   ```

3. **Audio Format Event** - Sent when playback starts

   Generated by [`src/ws_sink.rs`](../src/ws_sink.rs) in the `start()` method:

   ```json
   {
     "type": "audio_format",
     "device_id": "device-uuid",
     "data": {
       "sample_rate": 44100,
       "channels": 2,
       "bit_depth": 16,
       "format": "S16"
     }
   }
   ```

4. **Audio Data Event** - Sent continuously during playback

   Generated by [`src/ws_sink.rs`](../src/ws_sink.rs) in the `write()` method:

   ```json
   {
     "type": "audio_data",
     "device_id": "device-uuid",
     "data": {
       "format": "pcm_s16le",
       "encoded": "base64-encoded-audio-data",
       "packet_type": "samples"
     }
   }
   ```

5. **Audio Stream Stopped Event**

   Generated by [`src/ws_sink.rs`](../src/ws_sink.rs) in the `stop()` method:

   ```json
   {
     "type": "audio_stream_stopped",
     "device_id": "device-uuid",
     "data": {}
   }
   ```

> [!TIP]
> - Player event handling: [`src/spotify.rs`](../src/spotify.rs) lines 124-176
> - Sink event callbacks: [`src/spotify.rs`](../src/spotify.rs) lines 99-116
> - Audio format/data events: [`src/ws_sink.rs`](../src/ws_sink.rs)
> - The event format is not yet standardized and may change in future versions

### Command Management

Commands are routed through a command manager pattern implemented in [`src/command_manager.rs`](../src/command_manager.rs).

The `CommandManager` uses a trait-based handler system:

1. **`CommandHandler` trait** - All command handlers implement this interface
2. **Handler implementations** - Each command has a dedicated handler (e.g., `PlayCommandHandler`, `ShuffleCommandHandler`)
3. **`CommandManager::execute()`** - Routes commands to their appropriate handlers

This design provides a clean separation between:
- Command parsing (in `commands.rs`)
- Command routing (in `command_manager.rs`)
- Command execution (handlers in `command_manager.rs` calling `SpotifyClient` methods)

> [!TIP]
> - Command manager: [`src/command_manager.rs`](../src/command_manager.rs)
> - The `impl_simple_handler!` macro generates boilerplate for parameter-less commands
> - More complex commands like `Shuffle` and `SetVolume` have custom handler implementations

## Server

The server implementation is based on the [Warp](https://github.com/seanmonstar/warp) web framework, which provides async WebSocket support built on Tokio.

### Architecture

The server is implemented in [`src/server.rs`](../src/server.rs) with the following components:

#### `SpotifyServer`

The main server struct that:
- Manages the WebSocket route at `/ws`
- Spawns a handler for each client connection
- Contains a `CommandManager` for routing commands

#### Connection Handling

When a client connects:

1. **WebSocket split** - The connection is split into sender and receiver channels
2. **MPSC channel** - An unbounded channel is created for sending messages to the client
3. **Stream forwarding** - A task forwards messages from the MPSC channel to the WebSocket
4. **Message processing** - Incoming messages are processed by `process_ws_message()`

#### `ConnectionState`

Each WebSocket connection has its own `ConnectionState` that maintains:
- A `HashMap<String, SpotifyClient>` mapping device IDs to Spotify client instances
- This allows multiple Spotify Connect devices per WebSocket connection

### Current Limitations

1. **No Authentication** - Any client can connect and create devices
2. **No Authorization** - No control over which devices a client can access
3. **Messy Session Management**:
   - Device lifecycle is tied to WebSocket connection
   - No persistence of device state
   - No cleanup of disconnected devices
4. **Single Port** - Server listens only on localhost:8888 (configurable via CLI)
5. **No Rate Limiting** - Commands are processed without throttling
6. **No Connection Limits** - Unlimited concurrent connections allowed

### Configuration

Server configuration is minimal and done via CLI arguments (defined in [`src/main.rs`](../src/main.rs)):
- **Port** (`-p`, `--port`): WebSocket port (default: 8888)

The server always binds to `127.0.0.1` (localhost only).

> [!TIP]
> - Server implementation: [`src/server.rs`](../src/server.rs)
> - Entry point and CLI: [`src/main.rs`](../src/main.rs)
> - The server uses `warp::ws()` for WebSocket support
> - CORS is enabled for all origins via `warp::cors().allow_any_origin()`
> - Each connection is handled in a separate async task

## Librespot

[Librespot](https://github.com/librespot-org/librespot) is an open-source Spotify Connect client library written in Rust. It's the foundation that Blockyspot builds upon.

### Why Librespot?

Librespot provides:
- **Full Spotify Connect protocol implementation** - Allows devices to appear in Spotify's device list
- **Audio streaming** - Decodes and streams audio from Spotify's servers
- **Playback control** - Handles all Spotify Connect commands (play, pause, seek, etc.)
- **Authentication** - Manages OAuth tokens and session credentials
- **Caching** - Stores credentials and audio data for offline playback

### What We Use from Librespot

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

### What We Build On Top

Librespot provides the protocol and playback, but we add:

1. **Remote Control API** - WebSocket interface for external applications
2. **Custom Audio Sink** - Stream audio data over WebSockets (see [The Sink](#the-sink))
3. **Multi-Device Support** - Multiple Spotify sessions per server instance
4. **Event Broadcasting** - Real-time notifications to connected clients
5. **Command Abstraction** - Higher-level API over raw Spirc calls

### Initialization Flow

When a `CreateDevice` command is received ([`src/spotify.rs`](../src/spotify.rs) lines 59-197):

1. **Session Creation** - A Librespot `Session` is created with cache and config
2. **Credentials** - Access token is used to create `Credentials`
3. **Audio Pipeline** - `Player` is created with our custom `WebSocketSink`
4. **Event Handlers** - Callbacks are registered for player and sink events
5. **Spirc Setup** - The Connect protocol handler is initialized and started
6. **Event Tasks** - Background tasks are spawned to forward events to WebSocket clients

### Librespot Configuration

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

## The Sink

The audio sink is a critical component that intercepts audio data from Librespot's decoder and streams it to WebSocket clients. It's implemented in [`src/ws_sink.rs`](../src/ws_sink.rs).

### What is a Sink?

In Librespot's architecture, a "sink" is the audio output backend. It implements the `Sink` trait from `librespot::playback::audio_backend`, which defines how decoded audio should be output. Common sinks include ALSA (Linux audio), PortAudio (cross-platform), or file output.

Blockyspot provides a **custom sink** that instead of outputting to speakers, encodes audio and sends it over WebSockets.

### WebSocketSink Implementation

The `WebSocketSink` struct implements the `Sink` trait with three key methods:

#### 1. `start()`

Called when playback begins. It:
- Clears the internal buffer
- Sends an `audio_format` event to the client with sample rate, channels, and bit depth
- Marks the sink as active

Example message sent:
```json
{
  "type": "audio_format",
  "device_id": "uuid",
  "data": {
    "sample_rate": 44100,
    "channels": 2,
    "bit_depth": 16,
    "format": "S16"
  }
}
```

#### 2. `write(packet, converter)`

Called continuously during playback with audio data. It:
- Receives `AudioPacket` (either `Samples` or `Raw` data)
- Accumulates samples in an internal buffer
- When buffer reaches `chunk_size` (4410 samples = ~100ms), sends data to client
- Converts audio to S16 format and base64 encodes it
- Sends an `audio_data` event with the encoded chunk

Example message sent:
```json
{
  "type": "audio_data",
  "device_id": "uuid",
  "data": {
    "format": "pcm_s16le",
    "encoded": "base64_encoded_audio_data",
    "packet_type": "samples"
  }
}
```

#### 3. `stop()`

Called when playback stops. It:
- Clears the buffer
- Marks the sink as inactive
- Sends an `audio_stream_stopped` event

### Audio Pipeline

The complete audio pipeline in Blockyspot:

```
Spotify Servers
    ↓
Librespot Decoder (Vorbis/AAC)
    ↓
AudioPacket (f64 samples)
    ↓
WebSocketSink Buffer
    ↓
Converter (f64 → s16)
    ↓
Base64 Encoding
    ↓
WebSocket Message
    ↓
Client Application
```

### Buffering and Timing

The sink uses a buffer-based approach:
- **Buffer Size**: 4410 samples (approximately 100ms at 44.1kHz stereo)
- **Timing Control**: Enforces ~100ms between sends to avoid overwhelming clients
- **Format**: Sends PCM S16LE (signed 16-bit little-endian)

This buffering ensures:
- Efficient network usage (batching small packets)
- Smooth playback on client side
- Reasonable latency (~100ms)

### Creating the Sink

The sink is created via a factory function [`create_ws_sink()`](../src/ws_sink.rs) that:
- Takes an MPSC sender channel for WebSocket messages
- Takes an `AudioFormat` specification
- Takes a device ID for event tagging
- Returns a boxed `Sink` trait object

This factory is used in [`src/spotify.rs`](../src/spotify.rs) when initializing the Librespot player (lines 81-82).

### Key Design Decisions

1. **F64 → S16 Conversion** - Librespot outputs f64 samples, but S16 is more bandwidth-efficient
2. **Base64 Encoding** - Binary audio data is encoded for JSON transport
3. **Chunk-based Streaming** - Rather than per-sample, audio is sent in ~100ms chunks
4. **MPSC Channel** - Decouples audio processing from WebSocket I/O
5. **Device ID Tagging** - Each audio message is tagged with device_id for multi-device support

> [!TIP]
> - Sink implementation: [`src/ws_sink.rs`](../src/ws_sink.rs)
> - The `WebSocketSink` struct implements the Librespot `Sink` trait
> - Audio data flows through an MPSC channel to avoid blocking the audio thread
> - The `send_buffer()` method (lines 53-105) handles conversion and transmission
> - Rate limiting is implemented using `last_send_time` and `std::thread::sleep`
> - The sink is integrated into Librespot in [`src/spotify.rs`](../src/spotify.rs) via the `sink_builder` closure
