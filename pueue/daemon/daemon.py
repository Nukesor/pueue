import os
import fcntl
import pickle
import select
import subprocess

from helper import readQueue, writeQueue
from helper import getSocketName, createDir
from helper import getDaemonSocket


def daemonMain():
    # Create config dir, if not existing
    createDir()
    # Create daemon socket
    daemon = getDaemonSocket()
    # Get current Queue
    queue = readQueue()

    # Daemon states
    paused = False

    address = None
    process = None
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
                instruction = clientSocket.recv(8192)
                if instruction is not -1:
                    command = pickle.loads(instruction)
                    if command['mode'] == 'add':

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
                        key = command['key']
                        if key not in queue:
                            # Send error message to client in case there exists no such key
                            response = pickle.dumps('No command with key #' + str(key), -1)
                            clientSocket.send(response)
                        else:
                            # Delete command from queue, save the queue and send response to client
                            del queue[key]
                            writeQueue(queue)
                            response = pickle.dumps('Command #'+str(key)+' removed', -1)
                            clientSocket.send(response)
                        # Socket cleanup
                        read_list.remove(clientSocket)
                        clientSocket.close()

                    elif command['mode'] == 'show':
                        output = []
                        if command['index'] == 'all':
                            if len(queue) > 0:
                                output = queue
                            else:
                                output = 'Queue is empty'
                        elif command['index'] == 'current':
                            if process is not None:
                                while True:
                                    line = process.stdout.readline()
                                    if not line:
                                        break
                                    else:
                                        output.append(line)
                            else:
                                output = 'No process running right now'

                        response = pickle.dumps(output, -1)
                        clientSocket.send(response)
                        # Socket cleanup
                        read_list.remove(clientSocket)
                        clientSocket.close()

                    elif command['mode'] == 'EXIT':
                        print('Shutting down pueue daemon')
                        break

                    elif command['mode'] == 'KILL':
                        if (process is not None):
                            process.poll()
                            if process.returncode is None:
                                process.terminate()
                                answer = 'Send terminate to process'
                            else:
                                answer = "Process just terminated on it's own"
                        else:
                            answer = 'No process running'
                        response = pickle.dumps(answer, -1)
                        clientSocket.send(response)
                        read_list.remove(clientSocket)
                        clientSocket.close()

        if process is not None:
            process.poll()
            if process.returncode is not None:
                if process.returncode is not 0:
                    print(process.returncode)
                    print('We need an error log')
                queue.pop(min(queue.keys()), None)
                writeQueue(queue)
                process = None

        elif not paused:
            if (len(queue) > 0):
                nextItem = queue[min(queue.keys())]
                process = subprocess.Popen(
                        nextItem['command'],
                        shell=True,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.PIPE,
                        universal_newlines=True,
                        cwd=nextItem['path'])
                fd = process.stdout.fileno()
                fl = fcntl.fcntl(fd, fcntl.F_GETFL)
                fcntl.fcntl(fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)

    os.remove(getSocketName())
