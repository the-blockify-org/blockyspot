import socket
import json
import sys
import uuid

def send_command(sock, command):
    sock.send((json.dumps(command) + '\n').encode())
    response = sock.recv(1024).decode()
    return json.loads(response)

def main():
    # Connect to the server
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(('localhost', 8888))
    
    # Store the current device ID
    current_device_id = None

    try:
        while True:
            print("\nAvailable commands:")
            print("1. Create new device (requires Spotify token)")
            print("2. Play track (requires track ID)")
            print("3. Pause")
            print("4. Resume")
            print("5. Stop")
            print("6. Disconnect current device")
            print("7. Exit")
            
            choice = input("\nEnter command number: ")
            
            if choice == '1':
                token = input("Enter your Spotify token: ")
                device_id = str(uuid.uuid4())
                device_name = input("Enter device name (or press Enter for default): ").strip()
                cmd = {
                    "Connect": {
                        "token": token,
                        "device_id": device_id,
                        "device_name": device_name if device_name else None
                    }
                }
                response = send_command(sock, cmd)
                if response["success"]:
                    current_device_id = device_id
                    print(f"\nCreated device with ID: {device_id}")
            elif not current_device_id:
                print("\nError: No active device. Please create a device first.")
                continue
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
                    "Disconnect": {
                        "device_id": current_device_id
                    }
                }
                response = send_command(sock, cmd)
                if response["success"]:
                    current_device_id = None
            elif choice == '7':
                if current_device_id:
                    cmd = {
                        "Disconnect": {
                            "device_id": current_device_id
                        }
                    }
                    send_command(sock, cmd)
                break
            else:
                print("Invalid choice")
                continue
                
            if choice != '1' and choice != '6' and choice != '7':  # Already handled these cases
                response = send_command(sock, cmd)
            print("\nServer response:", json.dumps(response, indent=2))
            
    except KeyboardInterrupt:
        print("\nExiting...")
        if current_device_id:
            try:
                cmd = {
                    "Disconnect": {
                        "device_id": current_device_id
                    }
                }
                send_command(sock, cmd)
            except:
                pass
    except Exception as e:
        print(f"\nError: {e}")
    finally:
        sock.close()

if __name__ == "__main__":
    main() 