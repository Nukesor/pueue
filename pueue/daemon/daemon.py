import os
import time
import pickle
import select
import subprocess

from pueue.helper.files import createDir, createLogDir, getStdoutDescriptor
from pueue.helper.socket import getSocketName, getDaemonSocket


class Daemon():
    def __init__(self):
        # Create config dir, if not existing
        self.queueFolder = createDir()
        self.logDir = createLogDir()

        self.readQueue()
        self.socket = getDaemonSocket()
        if len(self.queue) != 0:
            self.nextKey = max(self.queue.keys()) + 1
            self.readLog(False)
            self.paused = True
        else:
            self.nextKey = 0
            self.paused = False
            self.readLog(True)

        self.active = True
        self.stopped = False

        self.current = None
        self.currentKey = None

        # Daemon states
        self.clientAddress = None
        self.clientSocket = None
        self.process = None
        self.read_list = [self.socket]

        #Stdout/Stderr management
        self.stdout = getStdoutDescriptor()

    def respondClient(self, answer):
        response = pickle.dumps(answer, -1)
        self.clientSocket.send(response)
        self.read_list.remove(self.clientSocket)
        self.clientSocket.close()

    def main(self):
        while self.active:
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
                        try:
                            command = pickle.loads(instruction)
                        except EOFError:
                            print('Received message is incomplete, dropping received data.')
                            self.read_list.remove(self.clientSocket)
                            self.clientSocket.close()

                            command = {}
                            command['mode'] = ''

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
                            self.respondClient('Pueue daemon shutting down')
                            self.active = False
                            break

            # Check if current process terminated
            if self.process is not None:
                self.process.poll()
                if self.process.returncode is not None:
                    if not self.stopped:
                        output, error_output = self.process.communicate()
                        self.stdout.seek(0)
                        output = self.stdout.read().replace('\n','\n    ')
                        self.log[min(self.queue.keys())] = self.queue[min(self.queue.keys())]
                        self.log[min(self.queue.keys())]['stderr'] = error_output
                        self.log[min(self.queue.keys())]['stdout'] = output
                        self.log[min(self.queue.keys())]['returncode'] = self.process.returncode
                        self.queue.pop(min(self.queue.keys()), None)
                        self.writeQueue()
                        self.writeLog()
                    self.process = None

            # Start next Process
            elif not self.paused:
                if (len(self.queue) > 0):
                    self.currentKey = min(self.queue.keys())
                    next_item = self.queue[self.currentKey]
                    self.stdout.seek(0)
                    self.stdout.truncate()
                    self.process = subprocess.Popen(
                            next_item['command'],
                            shell=True,
                            stdout=self.stdout,
                            stderr=subprocess.PIPE,
                            universal_newlines=True,
                            cwd=next_item['path'])

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
        home = os.path.expanduser('~')
        queuePath = home+'/.pueue/queue'
        queueFile = open(queuePath, 'wb+')
        try:
            pickle.dump(self.queue, queueFile, -1)
        except:
            print('Error while writing to queue file. Wrong file permissions?')
        queueFile.close()

    def readLog(self, rotate=False):
        logPath = self.queueFolder + '/queue.picklelog'
        if os.path.exists(logPath):
            logFile = open(logPath, 'rb')
            try:
                self.log = pickle.load(logFile)
            except:
                print('Log file corrupted, deleting old log')
                os.remove(logPath)
                self.log = {}
            logFile.close()
        else:
            self.log = {}

        if rotate:
            self.writeLog(True)
            os.remove(logPath)
            self.log = {}
            self.writeLog()

    def writeLog(self, rotate=False):

        if rotate:
            timestamp = time.strftime('%Y%m%d-%H%M')
            logPath = self.logDir + '/queue-' + timestamp + '.log'
        else:
            logPath = self.logDir + '/queue.log'

        picklelogPath = self.queueFolder + '/queue.picklelog'
        picklelogFile = open(picklelogPath, 'wb+')
        if os.path.exists(logPath):
            os.remove(logPath)
        logFile = open(logPath, 'w')
        logFile.write('Pueue log for executed Commands: \n \n')
        for key in self.log:
            try:
                logFile.write('Command #{} exited with returncode {}:  "{}" \n'.format(key, self.log[key]['returncode'], self.log[key]['command']))
                logFile.write('Path: {} \n'.format(self.log[key]['path']))
                if self.log[key]['stderr']:
                    logFile.write('Stderr output: \n    {}\n'.format(self.log[key]['stderr']))
                    logFile.write()
                logFile.write('Stdout output: \n    {}'.format(self.log[key]['stdout']))
                logFile.write('\n')
            except:
                print('Error while writing to log file. Wrong file permissions?')
        try:
            pickle.dump(self.log, picklelogFile, -1)
        except:
            print('Error while writing to picklelog file. Wrong file permissions?')
        logFile.close()
        picklelogFile.close()

    def executeAdd(self, command):
        # Add command to queue and save it
        self.queue[self.nextKey] = command
        self.nextKey += 1
        self.writeQueue()
        return 'Command added'

    def executeRemove(self, command):
        key = command['key']
        if key not in self.queue:
            # Send error answer to client in case there exists no such key
            answer = 'No command with key #' + str(key)
        else:
            # Delete command from queue, save the queue and send response to client
            if not self.paused and key == self.currentKey:
                answer = "Can't remove currently running process, please stop the process before removing it."
            else:
                del self.queue[key]
                self.writeQueue()
                answer = 'Command #{} removed'.format(key)
        return answer

    def executeSwitch(self, command):
        first = command['first']
        second = command['second']
        # Send error answer to client in case there exists no such key
        if first not in self.queue:
        # Send error answer to client in case there exists no such key
            answer = 'No command with key #' + str(first)
        elif second not in self.queue:
        # Send error answer to client in case there exists no such key
            answer = 'No command with key #' + str(second)
        else:
            # Delete command from queue, save the queue and send response to client
            if not self.paused and (first == self.currentKey or second == self.currentKey):
                answer = "Can't switch currently running process, please stop the process before switching it."
            else:
                tmp = self.queue[second].copy()
                self.queue[second] = self.queue[first].copy()
                self.queue[first] = tmp
                answer = 'Command #{} and #{} switched'.format(first, second)
        return answer

    def executeStatus(self, command):
        answer = {}
        data = []
        # Process status
        if (self.process is not None):
            self.process.poll()
            if self.process.returncode is None:
                answer['process'] = 'running'
        elif self.currentKey in self.log.keys():
            answer['process'] = 'finished'.format(self.log[self.currentKey]['returncode'])
        else:
            answer['process'] = 'no process'

        if self.paused:
            answer['status'] = 'paused'
        else:
            answer['status'] = 'running'
        if self.currentKey in self.log.keys():
            answer['current'] = self.log[self.currentKey]['returncode']
        else:
            answer['current'] = 'No exitcode'

        # Queue status
        if len(self.queue) > 0:
            data = self.queue
        else:
            data = 'Queue is empty'
        answer['data'] = data

        # Respond client
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
        self.writeLog()
        self.currentKey = None
        self.nextKey = 0
        answer = 'Reseting current queue'
        return answer

    def executeStart(self):
        if self.paused:
            self.paused = False
            answer = 'Daemon started'
        else:
            answer = 'Daemon alrady started'
        return answer

    def executePause(self):
        if not self.paused:
            self.paused = True
            answer = 'Daemon paused'
        else:
            answer = 'Daemon already paused'
        return answer

    def executeStop(self):
        if (self.process is not None):
            self.process.poll()
            if self.process.returncode is None:
                self.paused = True
                self.process.terminate()
                self.stopped = True
                answer = 'Terminating current process and pausing'
            else:
                answer = 'No process running, pausing daemon'
                self.paused = True
        else:
            answer = 'No process running, pausing daemon'
            self.paused = True
        return answer

    def executeKill(self):
        if (self.process is not None):
            self.process.poll()
            if self.process.returncode is None:
                self.paused = True
                self.process.kill()
                self.stopped = True
                answer = 'Sent kill to process and paused daemon'
            else:
                answer = "Process just terminated on it's own"
        else:
            answer = 'No process running, pausing daemon'
        return answer

