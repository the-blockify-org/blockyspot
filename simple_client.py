import asyncio
import json
import base64
import numpy as np
import sounddevice as sd
from websockets import connect
import logging
import sys
from collections import deque
import time

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

audio_config = None
stream = None

async def handle_message(message):
    global audio_config, stream

    data = json.loads(message)
    
    if "type" in data:
        msg_type = data["type"]
        
        if msg_type == "audio_format":
            audio_config = data["data"]
            logger.info("Received audio format:")
            for key, value in audio_config.items():
                logger.info(f"  {key}: {value}")
            
            stream = sd.OutputStream(
                samplerate=audio_config["sample_rate"],
                channels=audio_config["channels"],
                dtype=np.int16
            )
            stream.start()
            
        elif msg_type == "audio_data":
            if not audio_config or not stream:
                return
                
            try:
                decoded_data = base64.b64decode(data["data"]["encoded"])
                
                audio_array = np.frombuffer(decoded_data, dtype=np.int16)
                audio_array = audio_array.reshape(-1, audio_config["channels"])
                
                stream.write(audio_array)
            except Exception as e:
                logger.debug(f"Playback error: {e}")

async def main():
    uri = "ws://localhost:8888/ws"
    
    try:
        async with connect(uri) as websocket:
            logger.info("Connected to WebSocket server")
            
            initial_response = await websocket.recv()
            connection_data = json.loads(initial_response)
            device_id = connection_data["device_id"]
            logger.info(f"Device ID: {device_id}")
            
            token = input("Enter your Spotify token: ")
            
            connect_cmd = {
                "Connect": {
                    "token": token,
                    "device_id": device_id,
                    "device_name": "Python Audio Player"
                }
            }
            await websocket.send(json.dumps(connect_cmd))
            logger.info("Sent connect command")
            logger.info("Ready to play! Control playback from your phone/Spotify app")
            
            try:
                while True:
                    message = await websocket.recv()
                    await handle_message(message)
            except Exception as e:
                logger.error(f"Error in message loop: {e}")
            finally:
                if stream:
                    stream.stop()
                    stream.close()
    
    except Exception as e:
        logger.error(f"Connection error: {e}")
        if stream:
            stream.stop()
            stream.close()
        sys.exit(1)

if __name__ == "__main__":
    print("Required packages:")
    print("  pip install sounddevice numpy websockets")
    print("\nMake sure you have PortAudio installed:")
    print("  Windows: Should work out of the box")
    print("  macOS: brew install portaudio")
    print("  Linux: sudo apt-get install libportaudio2\n")
    
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nExiting...")
        if stream:
            stream.stop()
            stream.close()
    except Exception as e:
        logger.error(f"Fatal error: {e}")
        if stream:
            stream.stop()
            stream.close()
        sys.exit(1) 