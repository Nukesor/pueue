import sys, socket, time

from helper import getSocketName

def executeAdd(args):
    socketPath = getSocketName()
    try:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
        client.connect(socketPath)
    except:
        print("Error connecting to socket. Make sure the daemon is running")
        sys.exit(1)

    if args.command:
        print(args.command)
        client.send(bytes(args.command, 'UTF-8'))

