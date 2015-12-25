import os
import sys
import pickle
import select
import subprocess

from pueue.daemon.logs import writeLog
from pueue.helper.config import getConfig
from pueue.helper.socket import getSocketName, getDaemonSocket
from pueue.helper.files import createDir, createLogDir, getStdoutDescriptor


class Daemon():
    def __init__(self):
        # Create config dir, if not existing
        self.queueFolder = createDir()
        self.logDir = createLogDir()
        self.config = getConfig()

        self.readQueue()
        # Load previous queue
        # Reset queue if all jobs from last session are finished
        if self.getCurrentKey() is None:
            self.queue = {}
            self.writeQueue()
        self.socket = getDaemonSocket()

        # If there are still jobs in the queue the daemon might pause,
        # if this behaviour is defined in the config file.
        # The old logfile is beeing loaded as well.
        self.paused = False
        if len(self.queue) > 0:
            self.nextKey = max(self.queue.keys()) + 1
            self.readLog(False)
            if not self.config['default']['resumeAfterStart']:
                self.paused = True
        else:
            self.nextKey = 0
            self.readLog(True)

        self.active = True
        self.stopped = False

        # Variables for handling sockets and child process
        self.clientAddress = None
        self.clientSocket = None
        self.process = None
        self.read_list = [self.socket]

        # Stdout/Stderr management
        self.stdout = getStdoutDescriptor()

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
                            self.respondClient(self.executePause())

                        elif command['mode'] == 'stop':
                            self.respondClient(self.executeStop())

                        elif command['mode'] == 'kill':
                            self.respondClient(self.executeKill())

                        elif command['mode'] == 'STOPDAEMON':
                            self.respondClient({'message': 'Pueue daemon shutting down', 'status': 'success'})
                            # Kill current process and set active
                            # to False to stop while loop
                            self.active = False
                            self.executeKill()
                            break

            # Check if there is a running process
            if self.process is not None:
                # Poll process and check to check for termination
                self.process.poll()
                if self.process.returncode is not None:
                    if not self.stopped:
                        # Get std_out and err_out
                        output, error_output = self.process.communicate()
                        self.stdout.seek(0)
                        output = self.stdout.read().replace('\n', '\n    ')
                        currentKey = self.getCurrentKey()

                        # Mark queue entry as finished and save returncode
                        self.queue[currentKey]['returncode'] = self.process.returncode
                        if self.process.returncode != 0:
                            self.queue[currentKey]['status'] = 'errored'
                        else:
                            self.queue[currentKey]['status'] = 'done'

                        # Add outputs to log
                        self.logs[currentKey] = self.queue[currentKey]
                        self.logs[currentKey]['stderr'] = error_output
                        self.logs[currentKey]['stdout'] = output

                        # Pause Daemon, if it is configured to stop
                        if self.config['default']['stopAtError'] is True:
                            if self.process.returncode == 0:
                                self.paused = True

                        self.writeQueue()
                        self.log()
                    self.process = None

            # Start next Process
            elif not self.paused:
                if len(self.queue) > 0:
                    currentKey = self.getCurrentKey()
                    if currentKey is not None:
                        # Get instruction for next process
                        next_item = self.queue[currentKey]
                        self.stdout.seek(0)
                        self.stdout.truncate()
                        # Spawn subprocess
                        self.process = subprocess.Popen(
                            next_item['command'],
                            shell=True,
                            stdout=self.stdout,
                            stderr=subprocess.PIPE,
                            universal_newlines=True,
                            cwd=next_item['path']
                        )
                        self.queue[currentKey]['status'] = 'running'

        self.socket.close()
        os.remove(getSocketName())
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
            print('Errored while writing to queue file. Wrong file permissions?')
        queueFile.close()

    def readLog(self, rotate=False):
        # Read log of the previous session
        logPath = self.queueFolder + '/queue.picklelog'
        if os.path.exists(logPath):
            logFile = open(logPath, 'rb')
            try:
                self.logs = pickle.load(logFile)
            except:
                print('Log file corrupted, deleting old log')
                os.remove(logPath)
                self.logs = {}
            logFile.close()
        else:
            self.logs = {}

        # If rotate is True the logs will be rotated with a timestamp
        # and the logs will be resetted
        if rotate:
            self.log(True)
            self.logs = {}
            self.log()

    def log(self, rotate=False):
        # Log current log to a pickled file
        # We need this to preserve the log for reuse in a following session
        pickleLogPath = self.queueFolder + '/queue.picklelog'
        pickleLogFile = open(pickleLogPath, 'wb+')
        try:
            pickle.dump(self.logs, pickleLogFile, -1)
        except:
            print('Errored while writing to pickle log file. Wrong file permissions?')
        pickleLogFile.close()

        # If there is a finished process a
        # human readable log will be written
        if len(self.logs) > 0:
            writeLog(self.logDir, self.logs, rotate)

        return

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
                answer = {'message': "Can't remove currently running process, please stop the process before removing it.", 'status': 'error'}
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
                answer = {'message': "Can't switch currently running process, please stop the process before switching it.", 'status': 'error'}
            else:
                tmp = self.queue[second].copy()
                self.queue[second] = self.queue[first].copy()
                self.queue[first] = tmp
                answer = {'message': 'Command #{} and #{} switched'.format(first, second), 'status': 'success'}
        return answer

    def executeStatus(self, command):
        answer = {}
        data = []
        currentKey = self.getCurrentKey()
        # Get daemon status
        if self.paused:
            answer['status'] = 'paused'
            if currentKey in self.queue.keys():
                self.queue[currentKey]['status'] = 'queued'
        else:
            answer['status'] = 'running'

        # Get process status
        if self.process is not None:
            answer['process'] = 'running'
        else:
            answer['process'] = 'No running process'

        # Add current queue or a message, that queue is empty
        if len(self.queue) > 0:
            data = self.queue
        else:
            data = 'Queue is empty'
        answer['data'] = data

        return answer

    def executeReset(self):
        # Reset  queue
        self.queue = {}
        self.writeQueue()
        # Terminate current process
        if self.process is not None:
            self.process.terminate()
        # Rotate and reset Log
        self.readLog(True)
        self.log()
        self.nextKey = 0
        answer = {'message': 'Reseting current queue', 'status': 'success'}
        return answer

    def executeStart(self):
        # Start the daemon if in paused state
        if self.paused:
            self.paused = False
            answer = {'message': 'Daemon started', 'status': 'success'}
        else:
            answer = {'message': 'Daemon alrady running', 'status': 'omit'}
        return answer

    def executePause(self):
        # Pause the daemon running
        if not self.paused:
            self.paused = True
            answer = {'message': 'Daemon paused', 'status': 'success'}
        else:
            answer = {'message': 'Daemon already paused', 'status': 'omit'}
        return answer

    def executeStop(self):
        if (self.process is not None):
            # Check if process just exited at this moment
            self.process.poll()
            if self.process.returncode is None:
                # Pause daemon and terminate process
                self.paused = True
                self.process.terminate()
                self.stopped = True
                answer = {'message': 'Terminated current process and paused daemon', 'status': 'success'}
            else:
                answer = {'message': 'Process just finished, pausing daemon', 'status': 'omit'}
                self.paused = True
        else:
            # Only pausing daemon if no process is running
            answer = {'message': 'No process running, pausing daemon', 'status': 'omit'}
            self.paused = True
        return answer

    def executeKill(self):
        if (self.process is not None):
            # Check if process just exited at this moment
            self.process.poll()
            if self.process.returncode is None:
                # Pause daemon and kill process
                self.paused = True
                self.process.kill()
                self.stopped = True
                answer = {'message': 'Sent kill to process and paused daemon', 'status': 'success'}
            else:
                answer = {'message': "Process just terminated on it's own", 'status': 'omit'}
        else:
            # Only pausing daemon if no process is running
            answer = {'message': 'No process running, pausing daemon', 'status': 'omit'}
            self.paused = True
        return answer
