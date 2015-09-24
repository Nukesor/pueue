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
                        self.clientSocket, address = self.socket.accept()
                        self.read_list.append(self.clientSocket)
                    except:
                        print('Daemon rejected client')
                else:
                    instruction = self.clientSocket.recv(8192)
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
                            answer = []
                            if command['index'] == 'all':
                                if len(self.queue) > 0:
                                    answer = self.queue
                                else:
                                    answer = 'Queue is empty'
                            self.respondClient(answer)

                        elif command['mode'] == 'START':
                            if self.paused:
                                self.paused = False
                                answer = 'Daemon unpaused'
                            else:
                                answer = 'Daemon alrady unpaused'
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
                                    answer = 'Pueue daemon terminats current process and pauses'
                                else:
                                    answer = "No process running, pausing daemon"
                            else:
                                answer = "No process running, pausing daemon"
                            self.respondClient(answer)

                        elif command['mode'] == 'KILL':
                            if (self.process is not None):
                                self.process.poll()
                                if self.process.returncode is None:
                                    self.process.kill()
                                    answer = 'Send kill to process'
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
                        print(self.process.returncode)
                        print('We need an error log')
                    self.queue.pop(min(self.queue.keys()), None)
                    writeQueue(self.queue)
                    self.process = None

            elif not self.paused:
                if (len(self.queue) > 0):
                    nextItem = self.queue[min(self.queue.keys())]
                    self.process = subprocess.Popen(
                            nextItem['command'],
                            shell=True,
                            stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT,
                            universal_newlines=True,
                            cwd=nextItem['path'])

        self.socket.close()
        os.remove(getSocketName())
