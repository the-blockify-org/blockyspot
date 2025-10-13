# Protocol & Format

Blockyspot uses JSON-based messaging over WebSockets to communicate with clients. This guide describes the various message formats used in the protocol.

## Table of Contents
- [Command Format](#command-format)
- [Socket Communication](#socket-communication)
- [Event Format](#event-format)
- [Command Management](#command-management)

## Command Format

Commands sent to Blockyspot follow a standardized JSON structure defined in [`src/commands.rs`](../src/commands.rs).

### Command Message Structure

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

### Available Commands

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

## Socket Communication

The WebSocket server is implemented in [`src/server.rs`](../src/server.rs) and handles all client connections and message routing.

### Connection Flow

1. Client connects to `ws://localhost:8888/ws` (port configurable via CLI)
2. Server sends a connection response:
   ```json
   {
     "status": "Connected to server",
     "protocol_version": "0.1.1"
   }
   ```
3. Client can now send commands

### Response Format

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

### Session Management

The server maintains a `ConnectionState` per WebSocket connection, which tracks multiple devices (Spotify sessions) using a `HashMap<String, SpotifyClient>`. Each device is identified by a UUID generated when created.

> [!TIP]
> - Server implementation: [`src/server.rs`](../src/server.rs)
> - The `SpotifyServer::process_ws_message()` method handles all incoming messages
> - The `SpotifyServer::handle_client_connection()` method manages the WebSocket lifecycle
> - Protocol version is defined as `PROTOCOL_VERSION` constant

## Event Format

Events are sent from the server to the client to notify about changes in playback state, audio stream status, and other important occurrences.

### Event Types

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

## Command Management

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
