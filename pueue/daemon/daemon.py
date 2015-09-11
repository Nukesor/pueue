#import sys, os, time, socket, getpass, argparse
import os, socket, time

from helper import getSocketName

def daemonMain():
    # Creating Socket
    socketPath= getSocketName()
    if os.path.exists(socketPath):
        os.remove(socketPath)
    try:
        daemon = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
        daemon.bind(socketPath)
    except:
        print("Daemon couldn't bind to socket. Aborting")
        sys.exit(1)
    else:
        print("Daemon got socket")

    while True:
        message = daemon.recv(1024)
        print(message)
        if message == 'EXIT':
            daemon.exit()
            break
        time.sleep(1)

    daemon.close()
    os.remove(socketPath)

