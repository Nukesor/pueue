import sys
import socket
import time
import pickle

from helper import getSocketName

def executeStop(args):
    try:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
        client.connect(getSocketName())
    except:
        print("Error connecting to socket. Make sure the daemon is running")
        sys.exit(1)

    # Sending stop signal to daemon
    stopCommand = {'mode': 'EXIT'}
    data_string = pickle.dumps(stopCommand, -1)
    client.send(data_string)

