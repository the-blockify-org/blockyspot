import asyncio
import json
import sys
from websockets import connect
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

async def send_command(websocket, command):
    logger.debug(f"Sending command: {json.dumps(command)}")
    await websocket.send(json.dumps(command))

async def handle_message(message):
    data = json.loads(message)
    if "type" in data and data["type"] == "sink_event":
        status = data["data"]["status"]
        logger.info(f"🔊 Sink Event: {status}")
    else:
        print("\nServer response:", json.dumps(data, indent=2))

async def listen_for_messages(websocket):
    try:
        while True:
            message = await websocket.recv()
            await handle_message(message)
    except Exception as e:
        logger.error(f"Message listener error: {e}")

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