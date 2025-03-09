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

The server operates on WebSocket protocol (port 8888). When a client connects to `ws://localhost:8888/ws`, a connection is created where the client can execute commands like `CreateDevice` to start using the Spotify Connect device or commands like `Load` to start interacting with an specific device.

### Connection Flow

1. Client connects to `ws://localhost:8888/ws`
2. Server responds with a success or error message. In case of success, providing the protocol version.
```json
{
  "status": "Connected to server",
  "protocol_version": "1.0.0"
}
```
3. Now the client can start executing commands.

### Available Commands
- CreateDevice: Initialize a Spotify Connect device with an access token

### Command Format

All commands follow this JSON structure:

```json
{
    "command_type": "Name",
    "params": {
        "key": "value"
    }
}
```

For example:

```json
{
  "command_type": "CreateDevice",
  "params": {
    "device_name": "Blockify Boombox #2",
    "token": "yQRB....MEg3"
  }
}
```

### Server Responses

All server responses to commands follow this format:
```json
{
    "success": true/false,
    "message": "Response message",
    "data": {} // Optional additional data
}
```
## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 