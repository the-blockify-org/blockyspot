# The Audio Sink

The audio sink is a critical component that intercepts audio data from Librespot's decoder and streams it to WebSocket clients. It's implemented in [`src/ws_sink.rs`](../src/ws_sink.rs).

## Table of Contents
- [What is a Sink?](#what-is-a-sink)
- [WebSocketSink Implementation](#websocketsink-implementation)
- [Audio Pipeline](#audio-pipeline)
- [Buffering and Timing](#buffering-and-timing)
- [Creating the Sink](#creating-the-sink)
- [Key Design Decisions](#key-design-decisions)

## What is a Sink?

In Librespot's architecture, a "sink" is the audio output backend. It implements the `Sink` trait from `librespot::playback::audio_backend`, which defines how decoded audio should be output. Common sinks include ALSA (Linux audio), PortAudio (cross-platform), or file output.

Blockyspot provides a **custom sink** that instead of outputting to speakers, encodes audio and sends it over WebSockets.

## WebSocketSink Implementation

The `WebSocketSink` struct implements the `Sink` trait with three key methods:

### 1. `start()`

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

### 2. `write(packet, converter)`

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

### 3. `stop()`

Called when playback stops. It:
- Clears the buffer
- Marks the sink as inactive
- Sends an `audio_stream_stopped` event

## Audio Pipeline

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

## Buffering and Timing

The sink uses a buffer-based approach:
- **Buffer Size**: 4410 samples (approximately 100ms at 44.1kHz stereo)
- **Timing Control**: Enforces ~100ms between sends to avoid overwhelming clients
- **Format**: Sends PCM S16LE (signed 16-bit little-endian)

This buffering ensures:
- Efficient network usage (batching small packets)
- Smooth playback on client side
- Reasonable latency (~100ms)

## Creating the Sink

The sink is created via a factory function [`create_ws_sink()`](../src/ws_sink.rs) that:
- Takes an MPSC sender channel for WebSocket messages
- Takes an `AudioFormat` specification
- Takes a device ID for event tagging
- Returns a boxed `Sink` trait object

This factory is used in [`src/spotify.rs`](../src/spotify.rs) when initializing the Librespot player (lines 81-82).

## Key Design Decisions

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
