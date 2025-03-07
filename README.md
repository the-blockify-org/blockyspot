# Blockyspot

A Rust-based Spotify Connect device server that allows creating and managing multiple virtual Spotify Connect devices. Originally designed to enable Minecraft players to create personal Spotify Connect devices within the game.

## Features

- Create multiple virtual Spotify Connect devices
- Each device has a unique ID and custom name
- Control playback for each device independently
- Real-time bidirectional communication via WebSocket
- Automatic device ID generation
- Clean connection handling and resource management
- Python test client included

## Prerequisites

- Rust (latest stable version)
- Python 3.6+ (for test client)
- Spotify Premium account
- Spotify API access token
- Python websockets library (`pip install websockets`)

## Building

```bash
cargo build --release
```

## Running

1. Start the server:
```bash
cargo run --release
```

2. Run the test client:
```bash
python test_client.py
```

## WebSocket Protocol

The server operates on WebSocket protocol (port 8888). When a client connects to `ws://localhost:8888/ws`, the server automatically generates and returns a unique device ID. This ID is used for all subsequent commands.

### Connection Flow

1. Client connects to `ws://localhost:8888/ws`
2. Server generates and sends a device ID:
```json
{
    "device_id": "generated_uuid",
    "message": "Connected to server. Use this device_id for future commands."
}
```
3. Client uses this device ID for all future commands

### Available Commands

- Connect: Initialize a Spotify Connect device with an access token
- Play: Play a specific track
- Pause: Pause playback
- Resume: Resume playback
- Stop: Stop playback
- GetCurrentTrack: Get information about the current track (coming soon)

### Command Format

All commands follow this JSON structure:

```json
{
    "Command_Name": {
        "device_id": "device_id_from_server",
        ...command specific fields...
    }
}
```

Example commands:

```json
// Connect to Spotify
{
    "Connect": {
        "token": "spotify_access_token",
        "device_id": "device_id_from_server",
        "device_name": "Optional Device Name"
    }
}

// Play a track
{
    "Play": {
        "device_id": "device_id_from_server",
        "track_id": "spotify_track_id"
    }
}
```

### Server Responses

All server responses follow this format:
```json
{
    "success": true/false,
    "message": "Response message",
    "data": {} // Optional additional data
}
```

## Project Structure

- `src/`
  - `main.rs`: Server entry point
  - `server.rs`: WebSocket server and command handling
  - `spotify.rs`: Spotify Connect device implementation
  - `commands.rs`: Command and response types
- `test_client.py`: Python test client
- `Cargo.toml`: Rust dependencies and project metadata

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 