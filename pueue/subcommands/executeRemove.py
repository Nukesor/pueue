import sys
import os
import socket
import time
import pickle

from helper import getSocketName

def executeRemove(args):
    try:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
        client.bind(socketPath)
        client.connect(getSocketName())
    except:
        print("Error connecting to socket. Make sure the daemon is running")
        sys.exit(1)

    if args.key:
        addCommand = {'mode': 'add', 'key': key}
        data_string = pickle.dumps(addCommand, -1)
        client.send(data_string)
        answer = client.recv(1024)
        print(pickle.loads(answer))
        client.close()

