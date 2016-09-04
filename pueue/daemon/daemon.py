import os
import sys
import pickle
import select
import signal
import subprocess

from pueue.daemon.logs import write_log, remove_old_logs
from pueue.helper.config import getConfig
from pueue.helper.socket import getSocketPath, createDaemonSocket
from pueue.helper.files import createConfigDir, createLogDir, getStdoutDescriptor, getStderrDescriptor


class Daemon():
    def __init__(self):
        # Create config dir, if not existing
        self.queueFolder = createConfigDir()
        self.logDir = createLogDir()
        self.config = getConfig()

        remove_old_logs(self.config['log']['logTime'], self.logDir)
        self.readQueue()
        # Load previous queue
        # Reset queue if all jobs from last session are finished
        if self.getCurrentKey() is None:
            # Rotate old log
            self.log(rotate=True)
            self.queue = {}
            # Remove old log file
            self.log()
            self.writeQueue()
        self.socket = createDaemonSocket()

        # If there are still jobs in the queue the daemon might pause,
        # if this behaviour is defined in the config file.
        # The old logfile is beeing loaded as well.
        self.paused = False
        if len(self.queue) > 0:
            self.nextKey = max(self.queue.keys()) + 1
            if not self.config['default']['resumeAfterStart']:
                self.paused = True
        else:
            self.nextKey = 0

        self.active = True
        self.stopped = False
        self.reset = False

        # Variables to get the current state of a process
        self.processStatus = 'No running process'

        # Variables for handling sockets and child process
        self.clientAddress = None
        self.clientSocket = None
        self.process = None
        self.read_list = [self.socket]

        # Stdout/Stderr management
        self.stdout = getStdoutDescriptor()
        self.stderr = getStderrDescriptor()

    def getCurrentKey(self):
        # Get the current key of the queue.
        # Returns None if no key is found.
        smallest = None
        for key in self.queue.keys():
            if self.queue[key]['status'] != 'done' and self.queue[key]['status'] != 'errored':
                if smallest is None or key < smallest:
                    smallest = key
        return smallest

    def respondClient(self, answer):
        # Generic function to send an answer to the client
        response = pickle.dumps(answer, -1)
        self.clientSocket.send(response)
        self.read_list.remove(self.clientSocket)
        self.clientSocket.close()

    def main(self):
        while self.active:
            # Check if there is a running process
            if self.process is not None:
                # Poll process and check to check for termination
                self.process.poll()
                if self.process.returncode is not None:
                    # If a process is terminated by `stop` or `kill`
                    # we want to queue it again instead closing it as failed.
                    if not self.stopped:
                        # Get std_out and err_out
                        output, error_output = self.process.communicate()
                        self.stdout.seek(0)
                        output = self.stdout.read().replace('\n', '\n    ')

                        self.stderr.seek(0)
                        error_output = self.stderr.read().replace('\n', '\n    ')
                        currentKey = self.getCurrentKey()

                        # Mark queue entry as finished and save returncode
                        self.queue[currentKey]['returncode'] = self.process.returncode
                        if self.process.returncode != 0:
                            self.queue[currentKey]['status'] = 'errored'
                        else:
                            self.queue[currentKey]['status'] = 'done'

                        # Add outputs to log
                        self.queue[currentKey]['stderr'] = error_output
                        self.queue[currentKey]['stdout'] = output

                        # Pause Daemon, if it is configured to stop
                        if self.config['default']['stopAtError'] is True and not self.reset:
                            if self.process.returncode == 0:
                                self.paused = True

                        self.writeQueue()
                        self.log()
                    self.process = None
                    self.processStatus = 'No running process'

            if self.reset:
                # Reset  queue
                self.queue = {}
                self.writeQueue()

                # Rotate and reset Log
                self.log(rotate=True)
                self.log()
                self.nextKey = 0
                self.reset = False

            # Start next Process
            if not self.paused and len(self.queue) > 0 and self.process is None:
                currentKey = self.getCurrentKey()
                if currentKey is not None:
                    # Get instruction for next process
                    next_item = self.queue[currentKey]
                    #
                    self.stdout.seek(0)
                    self.stdout.truncate()
                    self.stderr.seek(0)
                    self.stderr.truncate()
                    # Spawn subprocess
                    self.process = subprocess.Popen(
                        next_item['command'],
                        shell=True,
                        stdout=self.stdout,
                        stderr=self.stderr,
                        universal_newlines=True,
                        cwd=next_item['path']
                    )
                    self.queue[currentKey]['status'] = 'running'
                    self.processStatus = 'running'

            # Create list for waitable objects
            readable, writable, errored = select.select(self.read_list, [], [], 1)
            for socket in readable:
                if socket is self.socket:
                    # Listening for clients to connect.
                    # Client sockets are added to readlist to be processed.
                    try:
                        self.clientSocket, self.clientAddress = self.socket.accept()
                        self.read_list.append(self.clientSocket)
                    except:
                        print('Daemon rejected client')
                else:
                    # Trying to receive instruction from client socket
                    try:
                        instruction = self.clientSocket.recv(8192)
                    except EOFError:
                        print('Client died while sending message, dropping received data.')
                        instruction = -1

                    # Check for valid instruction
                    if instruction != -1:
                        # Check if received data can be unpickled.
                        # Instruction will be ignored if it can't be unpickled
                        try:
                            command = pickle.loads(instruction)
                        except EOFError:
                            print('Received message is incomplete, dropping received data.')
                            self.read_list.remove(self.clientSocket)
                            self.clientSocket.close()

                            command = {}
                            command['mode'] = ''

                        # Executing respective function depending on command mode
                        if command['mode'] == 'add':
                            self.respondClient(self.executeAdd(command))

                        elif command['mode'] == 'remove':
                            self.respondClient(self.executeRemove(command))

                        elif command['mode'] == 'switch':
                            self.respondClient(self.executeSwitch(command))

                        elif command['mode'] == 'status':
                            self.respondClient(self.executeStatus(command))

                        elif command['mode'] == 'reset':
                            self.respondClient(self.executeReset())

                        elif command['mode'] == 'start':
                            self.respondClient(self.executeStart())

                        elif command['mode'] == 'pause':
                            self.respondClient(self.executePause(command))

                        elif command['mode'] == 'stop':
                            self.respondClient(self.executeStop())

                        elif command['mode'] == 'kill':
                            self.respondClient(self.executeKill())

                        elif command['mode'] == 'STOPDAEMON':
                            self.respondClient({'message': 'Pueue daemon shutting down',
                                                'status': 'success'})
                            # Kill current process and set active
                            # to False to stop while loop
                            self.active = False
                            self.executeKill()
                            break

        self.socket.close()
        os.remove(getSocketPath())
        sys.exit(0)

    def readQueue(self):
        queuePath = self.queueFolder+'/queue'
        if os.path.exists(queuePath):
            queueFile = open(queuePath, 'rb')
            try:
                self.queue = pickle.load(queueFile)
            except:
                print('Queue file corrupted, deleting old queue')
                os.remove(queuePath)
                self.queue = {}
            queueFile.close()
        else:
            self.queue = {}

    def writeQueue(self):
        queuePath = self.queueFolder + '/queue'
        queueFile = open(queuePath, 'wb+')
        try:
            pickle.dump(self.queue, queueFile, -1)
        except:
            print('Error while writing to queue file. Wrong file permissions?')
        queueFile.close()

    def log(self, rotate=False):
        # If there is a finished process a
        # human readable log will be written
        write_log(self.logDir, self.queue, rotate)

    def executeAdd(self, command):
        # Add command to queue and save it
        self.queue[self.nextKey] = command
        self.nextKey += 1
        self.writeQueue()
        return {'message': 'Command added', 'status': 'success'}

    def executeRemove(self, command):
        key = command['key']
        if key not in self.queue:
            # Send error answer to client in case there exists no such key
            answer = {'message': 'No command with key #{}'.format(str(key)), 'status': 'error'}
        else:
            # Delete command from queue, save the queue and send response to client
            if not self.paused and key == self.getCurrentKey():
                answer = {
                    'message': "Can't remove currently running process, please stop the process before removing it.",
                    'status': 'error'
                }
            else:
                del self.queue[key]
                self.writeQueue()
                answer = {'message': 'Command #{} removed'.format(key), 'status': 'success'}
        return answer

    def executeSwitch(self, command):
        first = command['first']
        second = command['second']
        # Send error answer to client in case there exists no such key
        if first not in self.queue:
            # Send error answer to client in case there exists no such key
            answer = {'message': 'No command with key #{}'.format(str(first)), 'status': 'error'}
        elif second not in self.queue:
            # Send error answer to client in case there exists no such key
            answer = {'message': 'No command with key #{}'.format(str(second)), 'status': 'error'}
        else:
            # Delete command from queue, save the queue and send response to client
            currentKey = self.getCurrentKey()
            if not self.paused and (first == currentKey or second == currentKey):
                answer = {
                    'message': "Can't switch currently running process, please stop the process before switching it.",
                    'status': 'error'
                }
            else:
                tmp = self.queue[second].copy()
                self.queue[second] = self.queue[first].copy()
                self.queue[first] = tmp
                answer = {
                    'message': 'Command #{} and #{} switched'.format(first, second),
                    'status': 'success'
                }
        return answer

    def executeStatus(self, command):
        answer = {}
        data = []
        # Get daemon status
        if self.paused:
            answer['status'] = 'paused'
        else:
            answer['status'] = 'running'

        # Get process status
        answer['process'] = self.processStatus

        # Add current queue or a message, that queue is empty
        if len(self.queue) > 0:
            data = self.queue
            # Remove stderr and stdout output for transfer
            # Some outputs are way to big for the socket buffer
            # and this is not needed by the client
            for key, item in data.items():
                if 'stderr' in item:
                    del item['stderr']
                if 'stdout' in item:
                    del item['stdout']
        else:
            data = 'Queue is empty'
        answer['data'] = data

        return answer

    def executeReset(self):
        # Terminate current process

        if self.process is not None:
            try:
                self.process.terminate(timout=10)
            except:
                self.process.kill()
            self.process.wait()

        self.reset = True
        self.log(reset=True)

        answer = {'message': 'Reseting current queue', 'status': 'success'}
        return answer

    def executeStart(self):
        # Start the process if it is paused
        if self.process is not None and self.paused:
            os.kill(self.process.pid, signal.SIGCONT)
            currentKey = self.getCurrentKey()
            self.queue[currentKey]['status'] = 'running'
            self.processStatus = 'running'

        # Start the daemon if in paused state
        if self.paused:
            self.paused = False
            answer = {'message': 'Daemon started', 'status': 'success'}
        else:
            answer = {'message': 'Daemon already running', 'status': 'success'}
        return answer

    def executePause(self, command):
        # Pause the currently running process
        if self.process is not None and command['wait']:
            os.kill(self.process.pid, signal.SIGSTOP)
            currentKey = self.getCurrentKey()
            self.queue[currentKey]['status'] = 'paused'
            self.processStatus = 'paused'

        # Pause the daemon
        if not self.paused:
            self.paused = True
            answer = {'message': 'Daemon paused', 'status': 'success'}
        else:
            answer = {'message': 'Daemon already paused', 'status': 'success'}
        return answer

    def executeStop(self):
        if (self.process is not None):
            # Check if process just exited at this moment
            self.process.poll()
            if self.process.returncode is None:
                # Terminate process
                self.process.terminate()

                # Pause and stop daemon
                self.paused = True
                self.stopped = True

                # Set status of current process in queue back to `queued`
                currentKey = self.getCurrentKey()
                self.queue[currentKey]['status'] = 'queued'

                answer = {'message': 'Terminated current process and paused daemon',
                          'status': 'success'}
            else:
                # Only pausing daemon if the process just finished right now.
                self.paused = True
                answer = {'message': 'Process just finished, pausing daemon', 'status': 'success'}
        else:
            # Only pausing daemon if no process is running
            self.paused = True
            answer = {'message': 'No process running, pausing daemon', 'status': 'success'}
        return answer

    def executeKill(self):
        if (self.process is not None):
            # Check if process just exited at this moment
            self.process.poll()
            if self.process.returncode is None:
                # Kill process
                self.process.kill()

                # Pause and stop daemon
                self.paused = True
                self.stopped = True

                # Set status of current process in queue back to `queued`
                currentKey = self.getCurrentKey()
                self.queue[currentKey]['status'] = 'queued'

                answer = {'message': 'Sent kill to process and paused daemon', 'status': 'success'}
            else:
                # Only pausing daemon if the process just finished right now.
                self.paused = True
                answer = {'message': "Process just terminated on it's own", 'status': 'success'}
        else:
            # Only pausing daemon if no process is running
            self.paused = True
            answer = {'message': 'No process running, pausing daemon', 'status': 'success'}
        return answer
