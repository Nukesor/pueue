import os
import sys
import socket
import time
import pickle
import select

from helper import readQueue, writeQueue
from helper import getSocketName, createDir
from helper import getDaemonSocket

def daemonMain():
    createDir()
    daemon = getDaemonSocket()

    address = None
    clientSocket = None
    read_list = [daemon]
    while True:
        readable, writable, errored = select.select(read_list, [], [], 1)
        for s in readable:
            if s is daemon:
                try:
                    clientSocket, address = daemon.accept()
                    read_list.append(clientSocket)
                except:
                    print('Daemon rejected client')
            else:
                instruction, address = clientSocket.recvfrom(8192)
                if instruction is not -1:
                    command = pickle.loads(instruction)
                    print(command)
                    if command['mode'] == 'add':
                        # Get current Queue
                        queue = readQueue()

                        # Calculate next index for queue
                        if len(queue) != 0:
                            nextKey = max(queue.keys()) + 1
                        else:
                            nextKey = 0

                        # Add command to queue and save it
                        queue[nextKey] = command
                        writeQueue(queue)

                        # Respond client
                        response = pickle.dumps('Command added', -1)
                        clientSocket.send(response)

                        # Socket cleanup
                        read_list.remove(clientSocket)
                        clientSocket.close()

                    elif command['mode'] == 'remove':
                        # Get current Queue
                        queue = readQueue()
                        print(queue)
                        key = command['key']
                        if not key in queue:
                        # Send error message to client in case there exists no such key
                            response = pickle.dumps('No command with key #' + str(key), -1)
                            clientSocket.send(response)
                        else:
                        # Delete command from queue, save the queue and send response to client
                            del queue[key];
                            writeQueue(queue)
                            response = pickle.dumps('Command #'+str(key)+' removed', -1)
                            clientSocket.send(response)
                        # Socket cleanup
                        read_list.remove(clientSocket)
                        clientSocket.close()

                    elif command['mode'] == 'show':
                        # Get current Queue and send it to client
                        queue = readQueue()
                        response = pickle.dumps(queue, -1)
                        clientSocket.send(response)
                        # Socket cleanup
                        read_list.remove(clientSocket)
                        clientSocket.close()

                    elif command['mode'] == 'EXIT':
                        print('Shutting down pueue daemon')
                        break


    os.remove(socketPath)

