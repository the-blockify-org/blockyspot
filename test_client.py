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
    response = await websocket.recv()
    logger.debug(f"Received response: {response}")
    return json.loads(response)

async def main():
    uri = "ws://localhost:8888/ws"
    try:
        async with connect(uri) as websocket:
            logger.info("Connected to WebSocket server")
            
            initial_response = await websocket.recv()
            connection_data = json.loads(initial_response)
            current_device_id = connection_data["device_id"]
            logger.info(f"Received device ID: {current_device_id}")

            while True:
                print("\nAvailable commands:")
                print("1. Connect to Spotify (requires token)")
                print("2. Play track (requires track ID)")
                print("3. Pause")
                print("4. Resume")
                print("5. Stop")
                print("6. Get current track")
                print("7. Exit")
                
                choice = input("\nEnter command number: ")
                
                if choice == '1':
                    token = input("Enter your Spotify token: ")
                    device_name = input("Enter device name (or press Enter for default): ").strip()
                    cmd = {
                        "Connect": {
                            "token": token,
                            "device_id": current_device_id,
                            "device_name": device_name if device_name else None
                        }
                    }
                elif choice == '2':
                    track_id = input("Enter track ID (base62 format): ")
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
                    response = await send_command(websocket, cmd)
                    print("\nServer response:", json.dumps(response, indent=2))
                except Exception as e:
                    logger.error(f"Error sending command: {e}")
                    break

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