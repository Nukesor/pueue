#import sys, os, time, socket, getpass, argparse
import os, sys, socket, time, pickle, stat, select

from helper import getSocketName, readQueue, writeQueue, createDir

def daemonMain():
    # Creating Socket
    socketPath= getSocketName()
    if os.path.exists(socketPath):
        os.remove(socketPath)
    createDir()
#    try:
    daemon = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
    daemon.bind(socketPath)
    daemon.listen(5)
    #os.chmod(socketPath, stat.S_IRWXU)
    daemon.setblocking(0)
    print('test')
#    except:
#        print("Daemon couldn't bind to socket. Aborting")
#        sys.exit(1)
#    else:
#        print("Daemon got socket")

    read_list = [daemon]
    readable, writable, errored = select.select(read_list, [], [])
    while True:
        print('test2')
        for s in readable:
            if s is daemon:
                try:
                    clientSocket, address = daemon.accept()
                    instruction = daemon.recv(1024)
                except BlockingIOError:
                    instruction = False

        if instruction:
            command = pickle.loads(instruction)
            if command['mode'] == 'add':
                queue = readQueue()
                if len(queue) != 0:
                    nextKey = max(queue.keys()) + 1
                else:
                    nextKey = 0
                queue[nextKey] = command
                writeQueue(queue)
                response = pickle.dumps('Command added', -1)
                daemon.sendto(response, address)
            elif command['mode'] == 'remove':
                queue = readQueue()
                key = command['key']
                if not queue[key]:
                    response = pickle.dumps('No command with key #'+key, -1)
                    daemon.sendto(response, address)
                else:
                    del queue[key];
                    writeQueue(queue)
                    response = pickle.dumps('Command #'+key+' removed', -1)
                    daemon.sendto(response, address)

            elif command['mode'] == 'EXIT':
                print('Shutting down pueue daemon')
                break


        time.sleep(0.5)

    os.remove(socketPath)

