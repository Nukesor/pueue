import os
import sys
import stat
import socket
import pickle

from pueue.helper.files import getSocketPath


def receiveData(socket):
    # Receive message from daemon
    answer = socket.recv(8192)
    response = pickle.loads(answer)
    socket.close()
    return response


def processResponse(response):
    # Print it and exit with 1 if operation wasn't successful
    print(response['message'])
    if response['status'] != 'success':
        sys.exit(1)


def connectClientSocket():
    # Create Socket and exit with 1, if socket can't be created
    try:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        client.connect(getSocketPath())
    except:
        print("Error connecting to socket. Make sure the daemon is running")
        sys.exit(1)
    return client


def createDaemonSocket():
    removeDaemonSocket()
    socketPath = getSocketPath()
    # Create Socket and exit with 1, if socket can't be created
    try:
        daemon = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        daemon.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        daemon.bind(socketPath)
        daemon.setblocking(0)
        daemon.listen(0)
        # Set file permissions
        os.chmod(socketPath, stat.S_IRWXU)
    except:
        print("Daemon couldn't bind to socket. Aborting")
        sys.exit(1)
    return daemon


def removeDaemonSocket():
    # Check for old socket and delete it
    socketPath = getSocketPath()
    if os.path.exists(socketPath):
        os.remove(socketPath)
