import os
import sys
import stat
import socket
import getpass

def getSocketName():
    # Generating pid and socket path from username
    try:
        userName = getpass.getuser()
    except:
        print("Couldn't get username from getpass.getuser(), aborting")
        sys.exit(1)
    else:
        home = os.path.expanduser('~')
        queueFolder = home+'/.pueue'
        socketPath = queueFolder+"/pueueSocket@"+userName+".sock"
        return socketPath

def getClientSocket():
    try:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        client.connect(getSocketName())
    except:
        print("Error connecting to socket. Make sure the daemon is running")
        sys.exit(1)
    return client

def getDaemonSocket():
    # Creating Socket
    socketPath= getSocketName()
    if os.path.exists(socketPath):
        os.remove(socketPath)

    try:
        daemon = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        daemon.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        daemon.bind(socketPath)
        daemon.setblocking(0)
        daemon.listen(0)
        os.chmod(socketPath, stat.S_IRWXU)
    except:
        print("Daemon couldn't bind to socket. Aborting")
        sys.exit(1)
    else:
        print("Daemon got socket")
    return daemon

