import asyncio
import json
import sys
from websockets import connect
import logging
import base64
import wave
import io
import numpy as np
from datetime import datetime
import time
import struct

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Global audio format settings
current_audio_format = None
output_file = None
frames = []
last_write_time = None
samples_written = 0

def calculate_packet_duration(decoded_data):
    # Calculate duration of the packet in seconds
    # For S16LE format, each sample is 2 bytes and we have multiple channels
    bytes_per_frame = 2 * current_audio_format['channels']
    num_frames = len(decoded_data) // bytes_per_frame
    return num_frames / current_audio_format['sample_rate']

def should_write_packet(packet_duration):
    global last_write_time, samples_written
    
    current_time = time.time()
    
    if last_write_time is None:
        last_write_time = current_time
        return True
        
    # Calculate expected time based on samples written
    expected_duration = samples_written / current_audio_format['sample_rate']
    actual_duration = current_time - last_write_time
    
    # If we're writing too fast, wait
    if actual_duration < expected_duration:
        time.sleep(expected_duration - actual_duration)
    
    return True

def init_wave_file():
    global output_file, last_write_time, samples_written
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = f"output_{timestamp}.wav"
    output_file = wave.open(filename, 'wb')
    output_file.setnchannels(current_audio_format['channels'])
    output_file.setsampwidth(current_audio_format['bit_depth'] // 8)
    output_file.setframerate(current_audio_format['sample_rate'])
    last_write_time = None
    samples_written = 0
    logger.info(f"Created new WAV file: {filename}")

async def send_command(websocket, command):
    logger.debug(f"Sending command: {json.dumps(command)}")
    await websocket.send(json.dumps(command))

async def handle_message(message):
    global current_audio_format, output_file, samples_written
    data = json.loads(message)
    
    if "type" in data:
        msg_type = data["type"]
        if msg_type == "sink_event":
            status = data["data"]["status"]
            logger.info(f"🔊 Sink Event: {status}")
            if "Stopped" in status:
                samples_written = 0
                last_write_time = None
            
        elif msg_type == "player_event":
            event_type = data["data"]["event_type"]
            details = data["data"]["details"]
            if details:
                logger.info(f"🎵 Player Event: {event_type}")
                for key, value in details.items():
                    logger.info(f"  └─ {key}: {value}")
            else:
                logger.info(f"🎵 Player Event: {event_type}")
                
        elif msg_type == "audio_format":
            current_audio_format = data["data"]
            logger.info(f"📊 Audio Format:")
            for key, value in current_audio_format.items():
                logger.info(f"  └─ {key}: {value}")
            if output_file:
                output_file.close()
            init_wave_file()
            
        elif msg_type == "audio_data":
            if not current_audio_format:
                logger.warning("Received audio data before format information")
                return
                
            audio_data = data["data"]
            if audio_data["format"] == "pcm_s16le":
                # Decode base64 audio data
                decoded_data = base64.b64decode(audio_data["encoded"])
                packet_duration = calculate_packet_duration(decoded_data)
                
                if should_write_packet(packet_duration):
                    if output_file:
                        output_file.writeframes(decoded_data)
                        samples_written += len(decoded_data) // (2 * current_audio_format['channels'])
                
    else:
        # This is a command response
        print("\nServer response:", json.dumps(data, indent=2))

async def listen_for_messages(websocket):
    try:
        while True:
            message = await websocket.recv()
            await handle_message(message)
    except Exception as e:
        logger.error(f"Message listener error: {e}")
        if output_file:
            output_file.close()

async def main():
    uri = "ws://localhost:8888/ws"
    try:
        async with connect(uri) as websocket:
            logger.info("Connected to WebSocket server")
            
            initial_response = await websocket.recv()
            connection_data = json.loads(initial_response)
            current_device_id = connection_data["device_id"]
            logger.info(f"Received device ID: {current_device_id}")

            message_listener = asyncio.create_task(listen_for_messages(websocket))

            while True:
                print("\nAvailable commands:")
                print("1. Connect to Spotify (requires token)")
                print("2. Play track (requires track ID)")
                print("3. Pause")
                print("4. Resume")
                print("5. Stop")
                print("6. Get current track")
                print("7. Exit")
                
                try:
                    choice = await asyncio.get_event_loop().run_in_executor(
                        None, lambda: input("\nEnter command number: ")
                    )
                except EOFError:
                    break

                if choice == '1':
                    token = await asyncio.get_event_loop().run_in_executor(
                        None, lambda: input("Enter your Spotify token: ")
                    )
                    device_name = await asyncio.get_event_loop().run_in_executor(
                        None, lambda: input("Enter device name (or press Enter for default): ").strip()
                    )
                    cmd = {
                        "Connect": {
                            "token": token,
                            "device_id": current_device_id,
                            "device_name": device_name if device_name else None
                        }
                    }
                elif choice == '2':
                    track_id = await asyncio.get_event_loop().run_in_executor(
                        None, lambda: input("Enter track ID (base62 format): ")
                    )
                    cmd = {
                        "Play": {
                            "device_id": current_device_id,
                            "track_id": track_id
                        }
                    }
                elif choice == '3':
                    cmd = {
                        "Pause": {
                            "device_id": current_device_id
                        }
                    }
                elif choice == '4':
                    cmd = {
                        "Resume": {
                            "device_id": current_device_id
                        }
                    }
                elif choice == '5':
                    cmd = {
                        "Stop": {
                            "device_id": current_device_id
                        }
                    }
                elif choice == '6':
                    cmd = {
                        "GetCurrentTrack": {
                            "device_id": current_device_id
                        }
                    }
                elif choice == '7':
                    break
                else:
                    print("Invalid choice")
                    continue
                
                try:
                    logger.info(f"Sending command: {json.dumps(cmd)}")
                    await send_command(websocket, cmd)
                except Exception as e:
                    logger.error(f"Error sending command: {e}")
                    break

            message_listener.cancel()
            try:
                await message_listener
            except asyncio.CancelledError:
                pass

    except Exception as e:
        logger.error(f"Connection error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nExiting...")
    except Exception as e:
        logger.error(f"Fatal error: {e}")
        sys.exit(1) 