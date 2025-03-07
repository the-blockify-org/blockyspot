use librespot::playback::audio_backend::{Sink, SinkError, SinkResult, Open};
use librespot::playback::config::AudioFormat;
use librespot::playback::decoder::AudioPacket;
use librespot::playback::convert::Converter;
use tokio::sync::mpsc;
use warp::ws::Message;
use crate::server::WsResult;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// A Sink implementation that sends audio data to WebSocket clients
pub struct WebSocketSink {
    sender: mpsc::UnboundedSender<WsResult<Message>>,
    format: AudioFormat,
    is_active: bool,
}

impl Open for WebSocketSink {
    fn open(_: Option<String>, format: AudioFormat) -> Self {
        // This will be called with a None sender, but we'll replace it later
        // with the actual sender when creating the sink in SpotifyClient
        let (tx, _) = mpsc::unbounded_channel();
        
        Self {
            sender: tx,
            format,
            is_active: false,
        }
    }
}

impl Sink for WebSocketSink {
    fn start(&mut self) -> SinkResult<()> {
        self.is_active = true;
        
        // Get sample rate and channels based on the format
        let (sample_rate, channels) = match self.format {
            AudioFormat::F64 | AudioFormat::F32 | AudioFormat::S32 | 
            AudioFormat::S24 | AudioFormat::S24_3 | AudioFormat::S16 => {
                // Default values, these should be set properly when initializing the sink
                (44100, 2)
            }
        };
        
        // Send format information to clients
        let format_info = serde_json::json!({
            "type": "audio_format",
            "data": {
                "sample_rate": sample_rate,
                "channels": channels,
                "bit_depth": match self.format {
                    AudioFormat::S16 => 16,
                    AudioFormat::S24 | AudioFormat::S24_3 => 24,
                    AudioFormat::S32 => 32,
                    AudioFormat::F32 => 32,
                    AudioFormat::F64 => 64,
                },
                "format": format!("{:?}", self.format),
            }
        });
        
        if let Ok(msg) = serde_json::to_string(&format_info) {
            let _ = self.sender.send(Ok(Message::text(msg)));
        }
        
        Ok(())
    }
    
    fn stop(&mut self) -> SinkResult<()> {
        self.is_active = false;
        
        // Notify clients that audio stream has stopped
        let stop_msg = serde_json::json!({
            "type": "audio_stream_stopped",
            "data": {}
        });
        
        if let Ok(msg) = serde_json::to_string(&stop_msg) {
            let _ = self.sender.send(Ok(Message::text(msg)));
        }
        
        Ok(())
    }
    
    fn write(&mut self, packet: AudioPacket, converter: &mut Converter) -> SinkResult<()> {
        if !self.is_active {
            return Ok(());
        }
        
        let buffer = match &packet {
            AudioPacket::Samples(samples) => {
                // Convert the audio samples to S16LE format
                // Use the converter to convert f64 samples to s16
                let s16_samples = converter.f64_to_s16(samples);
                
                // Convert Vec<i16> to bytes (u8)
                let byte_len = s16_samples.len() * 2; // 2 bytes per i16
                let mut byte_buffer = vec![0u8; byte_len];
                
                for (i, &sample) in s16_samples.iter().enumerate() {
                    let bytes = sample.to_le_bytes();
                    byte_buffer[i * 2] = bytes[0];
                    byte_buffer[i * 2 + 1] = bytes[1];
                }
                
                byte_buffer
            },
            AudioPacket::Raw(raw_data) => {
                // For raw data, we just use it directly
                raw_data.clone()
            }
        };
        
        // Encode the audio data as base64 to send over WebSocket
        let encoded = BASE64.encode(&buffer);
        
        // Create a message with the audio data
        let audio_msg = serde_json::json!({
            "type": "audio_data",
            "data": {
                "format": "pcm_s16le",
                "encoded": encoded,
                "packet_type": match packet {
                    AudioPacket::Samples(_) => "samples",
                    AudioPacket::Raw(_) => "raw",
                },
            }
        });
        
        if let Ok(msg) = serde_json::to_string(&audio_msg) {
            if let Err(_) = self.sender.send(Ok(Message::text(msg))) {
                // Use a valid SinkError variant
                return Err(SinkError::NotConnected("Failed to send audio data to WebSocket clients".to_string()));
            }
        }
        
        Ok(())
    }
}

impl WebSocketSink {
    /// Set the WebSocket sender for this sink
    pub fn set_sender(&mut self, sender: mpsc::UnboundedSender<WsResult<Message>>) {
        self.sender = sender;
    }
    
    /// Create a new WebSocketSink with the given sender and audio format
    pub fn with_sender(
        sender: mpsc::UnboundedSender<WsResult<Message>>,
        format: AudioFormat
    ) -> Self {
        Self {
            sender,
            format,
            is_active: false,
        }
    }
}

/// A builder function that creates a new WebSocketSink
pub fn create_ws_sink(
    sender: mpsc::UnboundedSender<WsResult<Message>>,
    format: AudioFormat
) -> Box<dyn Sink> {
    Box::new(WebSocketSink::with_sender(sender, format))
} 