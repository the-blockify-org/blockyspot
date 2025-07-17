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

By default, the server runs on port 8888. You can specify a custom port using the `-p` or `--port` flag:

```bash
# Run on port 9000
cargo run --release -- --port 9000

# Or using the short form
cargo run --release -- -p 9000
```

2. Run the test client:
```bash
python test_client.py
```

## Command Line Options

The server supports the following command-line arguments:

- `-p, --port <PORT>`: Port to run the WebSocket server on (default: 8888)
- `-h, --help`: Show help information
- `-V, --version`: Show version information

### Examples

```bash
# Start server on default port (8888)
./blockyspot

# Start server on port 3000
./blockyspot --port 3000

# Start server on port 9999 using short form
./blockyspot -p 9999

# Show help
./blockyspot --help
```

## WebSocket Protocol

The server operates on WebSocket protocol (default port 8888, configurable via command line). When a client connects to `ws://localhost:<port>/ws`, a connection is created where the client can execute commands like `CreateDevice` to start using the Spotify Connect device or commands like `Load` to start interacting with an specific device.

### Connection Flow

1. Client connects to `ws://localhost:<port>/ws` (default port is 8888)
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