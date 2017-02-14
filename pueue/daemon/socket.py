import os
import sys
import stat
import socket


def create_socket(config_dir):
    """Create a socket for the daemon, depending on the directory location.

    Args:
        config_dir (str): The absolute path to the config directory used by the daemon.

    Returns:
        socket.socket: The daemon socket. Clients connect to this socket.
    """

    socket_path = os.path.join(config_dir, 'pueue.sock')
    # Create Socket and exit with 1, if socket can't be created
    try:
        if os.path.exists(socket_path):
            os.remove(socket_path)
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
