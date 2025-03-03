# Blockyspot

A Rust-based Spotify Connect device server that allows creating and managing multiple virtual Spotify Connect devices. Originally designed to enable Minecraft players to create personal Spotify Connect devices within the game.

## Features

- Create multiple virtual Spotify Connect devices
- Each device has a unique ID and custom name
- Control playback for each device independently
- Clean connection handling and resource management
- Simple TCP-based command protocol
- Python test client included

## Prerequisites

- Rust (latest stable version)
- Python 3.6+ (for test client)
- Spotify Premium account
- Spotify API access token

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

## Command Protocol

The server accepts JSON commands over TCP (port 8888). Each command must include a device ID except for the initial Connect command.

### Available Commands

- Connect: Create a new Spotify Connect device
- Play: Play a specific track
- Pause: Pause playback
- Resume: Resume playback
- Stop: Stop playback
- Disconnect: Remove the device

### Command Format

```json
{
    "Connect": {
        "token": "spotify_access_token",
        "device_id": "unique_device_id",
        "device_name": "Optional Device Name"
    }
}
```

## Project Structure

- `src/`
  - `main.rs`: Server entry point
  - `server.rs`: TCP server and command handling
  - `spotify.rs`: Spotify Connect device implementation
  - `commands.rs`: Command and response types
- `test_client.py`: Python test client
- `Cargo.toml`: Rust dependencies and project metadata

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 