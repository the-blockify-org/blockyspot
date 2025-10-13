# Server Architecture

The server implementation is based on the [Warp](https://github.com/seanmonstar/warp) web framework, which provides async WebSocket support built on Tokio.

## Table of Contents
- [Architecture](#architecture)
- [Current Limitations](#current-limitations)
- [Configuration](#configuration)

## Architecture

The server is implemented in [`src/server.rs`](../src/server.rs) with the following components:

### `SpotifyServer`

The main server struct that:
- Manages the WebSocket route at `/ws`
- Spawns a handler for each client connection
- Contains a `CommandManager` for routing commands

### Connection Handling

When a client connects:

1. **WebSocket split** - The connection is split into sender and receiver channels
2. **MPSC channel** - An unbounded channel is created for sending messages to the client
3. **Stream forwarding** - A task forwards messages from the MPSC channel to the WebSocket
4. **Message processing** - Incoming messages are processed by `process_ws_message()`

### `ConnectionState`

Each WebSocket connection has its own `ConnectionState` that maintains:
- A `HashMap<String, SpotifyClient>` mapping device IDs to Spotify client instances
- This allows multiple Spotify Connect devices per WebSocket connection

## Current Limitations

1. **No Authentication** - Any client can connect and create devices
2. **No Authorization** - No control over which devices a client can access
3. **Messy Session Management**:
   - Device lifecycle is tied to WebSocket connection
   - No persistence of device state
   - No cleanup of disconnected devices
4. **Single Port** - Server listens only on localhost:8888 (configurable via CLI)
5. **No Rate Limiting** - Commands are processed without throttling
6. **No Connection Limits** - Unlimited concurrent connections allowed

## Configuration

Server configuration is minimal and done via CLI arguments (defined in [`src/main.rs`](../src/main.rs)):
- **Port** (`-p`, `--port`): WebSocket port (default: 8888)

The server always binds to `127.0.0.1` (localhost only).

> [!TIP]
> - Server implementation: [`src/server.rs`](../src/server.rs)
> - Entry point and CLI: [`src/main.rs`](../src/main.rs)
> - The server uses `warp::ws()` for WebSocket support
> - CORS is enabled for all origins via `warp::cors().allow_any_origin()`
> - Each connection is handled in a separate async task
