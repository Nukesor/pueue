import os
import sys
import stat
import socket
import pickle



def receive_data(socket):
    # Receive message from daemon
    answer = socket.recv(1048576)
    response = pickle.loads(answer)
    socket.close()
    return response


def process_response(response):
    # Print it and exit with 1 if operation wasn't successful
    print(response['message'])
    if response['status'] != 'success':
        sys.exit(1)


def connect_client_socket(root_dir=None):
    # Determine config directory to find the daemon socket
    if root_dir is not None:
        config_dir = os.path.join(root_dir, '.config/pueue')
    else:
        config_dir = os.path.join(os.path.expanduser('~'), '.config/pueue')
    # Create Socket and exit with 1, if socket can't be created
    try:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        socket_path = os.path.join(config_dir, 'pueue.sock')
        client.connect(socket_path)
    except:
        print("Error connecting to socket. Make sure the daemon is running")
        sys.exit(1)
    return client


def create_daemon_socket(config_dir):
    socket_path = os.path.join(config_dir, 'pueue.sock')
    # Create Socket and exit with 1, if socket can't be created
    try:
        daemon = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        daemon.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        daemon.bind(socket_path)
        daemon.setblocking(0)
        daemon.listen(0)
        # Set file permissions
        os.chmod(socket_path, stat.S_IRWXU)
    except:
        print("Daemon couldn't bind to socket. Aborting")
        sys.exit(1)
    return daemon
