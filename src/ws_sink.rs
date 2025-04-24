use librespot::playback::audio_backend::{Sink, SinkError, SinkResult, Open};
use librespot::playback::config::AudioFormat;
use librespot::playback::decoder::AudioPacket;
use librespot::playback::convert::Converter;
use tokio::sync::mpsc;
use warp::ws::Message;
use crate::server::WsResult;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub struct WebSocketSink {
    sender: mpsc::UnboundedSender<WsResult<Message>>,
    format: AudioFormat,
    is_active: bool,
    buffer: Vec<f64>,
    chunk_size: usize,
    last_send_time: Option<std::time::Instant>,
    device_id: String,
}

impl Open for WebSocketSink {
    fn open(_: Option<String>, format: AudioFormat) -> Self {
        let (tx, _) = mpsc::unbounded_channel();
        
        Self {
            sender: tx,
            format,
            is_active: false,
            buffer: Vec::new(),
            chunk_size: 4410,
            last_send_time: None,
            device_id: String::new(),
        }
    }
}

impl WebSocketSink {
    pub fn set_sender(&mut self, sender: mpsc::UnboundedSender<WsResult<Message>>) {
        self.sender = sender;
    }
    
    pub fn with_sender(
        sender: mpsc::UnboundedSender<WsResult<Message>>,
        format: AudioFormat,
        device_id: String,
    ) -> Self {
        Self {
            sender,
            format,
            is_active: false,
            buffer: Vec::new(),
            chunk_size: 4410,
            last_send_time: None,
            device_id,
        }
    }

    fn send_buffer(&mut self, converter: &mut Converter) -> SinkResult<()> {
        use std::time::{Duration, Instant};

        if self.buffer.is_empty() {
            return Ok(());
        }

        let now = Instant::now();

        if let Some(last_send) = self.last_send_time {
            let elapsed = now.duration_since(last_send);
            let target_duration = Duration::from_millis(100);
            
            if elapsed < target_duration {
                std::thread::sleep(target_duration - elapsed);
            }
        }

        let s16_samples = converter.f64_to_s16(&self.buffer);
        
        let byte_len = s16_samples.len() * 2;
        let mut byte_buffer = vec![0u8; byte_len];
        
        for (i, &sample) in s16_samples.iter().enumerate() {
            let bytes = sample.to_le_bytes();
            byte_buffer[i * 2] = bytes[0];
            byte_buffer[i * 2 + 1] = bytes[1];
        }

        let encoded = BASE64.encode(&byte_buffer);
        
        let audio_msg = serde_json::json!({
            "type": "audio_data",
            "device_id":  &self.device_id,
            "data": {
                "format": "pcm_s16le",
                "encoded": encoded,
                "packet_type": "samples",
            }
        });

        if let Ok(msg) = serde_json::to_string(&audio_msg) {
            if let Err(_) = self.sender.send(Ok(Message::text(msg))) {
                return Err(SinkError::NotConnected("Failed to send audio data to WebSocket clients".to_string()));
            }
        }

        self.last_send_time = Some(now);
        self.buffer.clear();
        Ok(())
    }
}

impl Sink for WebSocketSink {
    fn start(&mut self) -> SinkResult<()> {
        self.is_active = true;
        self.buffer.clear();
        self.last_send_time = None;
        
        let (sample_rate, channels) = match self.format {
            AudioFormat::F64 | AudioFormat::F32 | AudioFormat::S32 | 
            AudioFormat::S24 | AudioFormat::S24_3 | AudioFormat::S16 => {
                (44100, 2)
            }
        };
        
        let format_info = serde_json::json!({
            "type": "audio_format",
            "device_id":  &self.device_id,
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
        self.buffer.clear();
        self.last_send_time = None;
        
        let stop_msg = serde_json::json!({
            "type": "audio_stream_stopped",
            "device_id":  &self.device_id,
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
        
        match &packet {
            AudioPacket::Samples(samples) => {
                self.buffer.extend_from_slice(samples);
                
                if self.buffer.len() >= self.chunk_size {
                    self.send_buffer(converter)?;
                }
            },
            AudioPacket::Raw(raw_data) => {
                let encoded = BASE64.encode(raw_data);
                let audio_msg = serde_json::json!({
                    "type": "audio_data",
                    "device_id":  &self.device_id,
                    "data": {
                        "format": "pcm_s16le",
                        "encoded": encoded,
                        "packet_type": "raw",
                    }
                });
                
                if let Ok(msg) = serde_json::to_string(&audio_msg) {
                    if let Err(_) = self.sender.send(Ok(Message::text(msg))) {
                        return Err(SinkError::NotConnected("Failed to send audio data to WebSocket clients".to_string()));
                    }
                }
            }
        }
        
        Ok(())
    }
}

pub fn create_ws_sink(
    sender: mpsc::UnboundedSender<WsResult<Message>>,
    format: AudioFormat,
    device_id: String,
) -> Box<dyn Sink> {
    Box::new(WebSocketSink::with_sender(sender, format, device_id))
} 