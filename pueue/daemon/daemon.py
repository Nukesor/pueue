import os
import pickle
import select
import subprocess

from pueue.helper.queue import readQueue, writeQueue
from pueue.helper.paths import createDir
from pueue.helper.socket import getSocketName, getDaemonSocket


class Daemon():
    def __init__(self):
        # Create config dir, if not existing
        createDir()
        self.queue = readQueue()
        self.socket = getDaemonSocket()

        # Daemon states
        self.paused = False
        self.clientAddress = None
        self.clientSocket = None
        self.process = None
        self.read_list = [self.socket]

    def respondClient(self, answer):
        response = pickle.dumps(answer, -1)
        self.clientSocket.send(response)
        self.read_list.remove(self.clientSocket)
        self.clientSocket.close()

    def main(self):
        while True:
            readable, writable, errored = select.select(self.read_list, [], [], 1)
            for s in readable:
                if s is self.socket:
                    try:
                        self.clientSocket, self.clientAddress = self.socket.accept()
                        self.read_list.append(self.clientSocket)
                    except:
                        print('Daemon rejected client')
                else:
                    try:
                        instruction = self.clientSocket.recv(8192)
                    except EOFError:
                        print('Client died while sending message, dropping received data.')
                        instruction = -1

                    if instruction is not -1:
                        command = pickle.loads(instruction)
                        if command['mode'] == 'add':

                            # Calculate next index for queue
                            if len(self.queue) != 0:
                                nextKey = max(self.queue.keys()) + 1
                            else:
                                nextKey = 0

                            # Add command to queue and save it
                            self.queue[nextKey] = command
                            writeQueue(self.queue)
                            self.respondClient('Command added')

                        elif command['mode'] == 'remove':
                            key = command['key']
                            if key not in self.queue:
                                # Send error answer to client in case there exists no such key
                                answer = 'No command with key #' + str(key)
                            else:
                                # Delete command from queue, save the queue and send response to client
                                del self.queue[key]
                                writeQueue(self.queue)
                                answer = 'Command #'+str(key)+' removed'
                                self.respondClient(answer)

                        elif command['mode'] == 'show':
                            answer = {}
                            data = []
                            # Process status
#                            if (self.process is not None):
#                                self.process.poll()
#                                if self.process.returncode is None:
#                                    answer['status'] = 'running'
#                                else:
#                                    answer['status'] = 'Exited with'+str(self.process.returncode)
#                            else:
#                                answer['status'] = 'no process'

                            # Queue status
                            if command['index'] == 'all':
                                if len(self.queue) > 0:
                                    data = self.queue
                                else:
                                    data = 'Queue is empty'
                            answer['data'] = data

                            # Respond client
                            self.respondClient(answer)

                        elif command['mode'] == 'START':
                            if self.paused:
                                self.paused = False
                                answer = 'Daemon started'
                            else:
                                answer = 'Daemon alrady started'
                            self.respondClient(answer)

                        elif command['mode'] == 'PAUSE':
                            if not self.paused:
                                self.paused = True
                                answer = 'Daemon paused'
                            else:
                                answer = 'Daemon already paused'
                            self.respondClient(answer)

                        elif command['mode'] == 'STOP':
                            if (self.process is not None):
                                self.process.poll()
                                if self.process.returncode is None:
                                    self.process.terminate()
                                    answer = 'Terminating current process and pausing'
                                else:
                                    answer = "No process running, pausing daemon"
                            else:
                                answer = "No process running, pausing daemon"
                            self.respondClient(answer)

                        elif command['mode'] == 'KILL':
                            if (self.process is not None):
                                self.process.poll()
                                if self.process.returncode is None:
                                    self.paused = True
                                    self.process.kill()
                                    answer = 'Sent kill to process'
                                else:
                                    answer = "Process just terminated on it's own"
                            else:
                                answer = 'No process running'
                            self.respondClient(answer)

                        elif command['mode'] == 'EXIT':
                            self.respondClient('Pueue daemon shutting down')
                            break

            if self.process is not None:
                self.process.poll()
                if self.process.returncode is not None:
                    if self.process.returncode is not 0:
                        output, error_output = self.process.communicate()
                        print('We need an error log')
                    self.queue.pop(min(self.queue.keys()), None)
                    writeQueue(self.queue)
                    self.process = None

            elif not self.paused:
                if (len(self.queue) > 0):
                    next_item = self.queue[min(self.queue.keys())]
                    self.process = subprocess.Popen(
                            next_item['command'],
                            shell=True,
                            stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE,
                            universal_newlines=True,
                            cwd=next_item['path'])

        self.socket.close()
        os.remove(getSocketName())
