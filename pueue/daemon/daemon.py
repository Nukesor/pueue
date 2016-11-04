import os
import sys
import pickle
import select
import signal
import subprocess
from copy import deepcopy
from datetime import datetime

from pueue.daemon.logs import write_log, remove_old_logs
from pueue.helper.config import get_config
from pueue.helper.socket import get_socket_path, create_daemon_socket
from pueue.helper.files import create_config_dir, create_log_dir, get_stdout_descriptor, get_stderr_descriptor


class Daemon():
    def __init__(self):
        # Create config dir, if not existing
        self.queueFolder = create_config_dir()
        self.logDir = create_log_dir()
        self.config = get_config()

        remove_old_logs(self.config['log']['logTime'], self.logDir)
        self.read_queue()
        # Load previous queue
        # Reset queue if all jobs from last session are finished
        if self.get_next_item() is None:
            # Rotate old log
            self.log(rotate=True)
            self.queue = {}
            # Remove old log file
            self.log()
            self.write_queue()
        self.socket = create_daemon_socket()

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
        self.stopping = False
        self.reset = False
        self.remove_current = False

        # Variables to get the current state of a process
        self.processStatus = 'No running process'

        # Variables for handling sockets and child process
        self.clientAddress = None
        self.clientSocket = None
        self.process = None
        self.read_list = [self.socket]

        # Stdout/Stderr management
        self.stdout = get_stdout_descriptor()
        self.stderr = get_stderr_descriptor()

    def get_next_item(self):
        # Get the next processable item of the queue.
        # Returns None if no key is found.
        smallest = None
        for key in self.queue.keys():
            if self.queue[key]['status'] == 'queued':
                if smallest is None or key < smallest:
                    smallest = key
        return smallest

    def respond_client(self, answer):
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
                    if not self.stopping:
                        # Get std_out and err_out
                        output, error_output = self.process.communicate()
                        self.stdout.seek(0)
                        output = self.stdout.read().replace('\n', '\n    ')

                        self.stderr.seek(0)
                        error_output = self.stderr.read().replace('\n', '\n    ')

                        # Mark queue entry as finished and save returncode
                        self.queue[self.current_key]['returncode'] = self.process.returncode
                        if self.process.returncode != 0:
                            self.queue[self.current_key]['status'] = 'errored'
                        else:
                            self.queue[self.current_key]['status'] = 'done'

                        # Add outputs to log
                        self.queue[self.current_key]['stdout'] = output
                        self.queue[self.current_key]['stderr'] = error_output
                        self.queue[self.current_key]['end'] = str(datetime.now().strftime("%H:%M"))

                        # Pause Daemon, if it is configured to stop
                        if self.config['default']['stopAtError'] is True and not self.reset:
                            if self.process.returncode != 0:
                                self.paused = True

                        self.write_queue()
                        self.log()
                    else:
                        # Process finally finished.
                        # Now we can set the status to paused.
                        self.paused = True
                        self.stopping = False
                        if self.remove_current is True:
                            self.remove_current = False
                            del self.queue[self.current_key]
                        else:
                            self.queue[self.current_key]['status'] = 'queued'

                    self.process = None
                    self.current_key = None
                    self.processStatus = 'No running process'

            if self.reset:
                # Rotate log
                self.log(rotate=True)

                # Reset  queue
                self.queue = {}
                self.write_queue()

                # Reset Log
                self.log()
                self.nextKey = 0
                self.reset = False

            # Start next Process
            if not self.paused and len(self.queue) > 0 and self.process is None:
                self.current_key = self.get_next_item()
                if self.current_key is not None:
                    # Get instruction for next process
                    next_item = self.queue[self.current_key]
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
                        stdin=subprocess.PIPE,
                        universal_newlines=True,
                        cwd=next_item['path']
                    )
                    self.queue[self.current_key]['status'] = 'running'
                    self.queue[self.current_key]['start'] = str(datetime.now().strftime("%H:%M"))
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
                        instruction = self.clientSocket.recv(1048576)
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
                            self.respond_client(self.execute_add(command))

                        elif command['mode'] == 'remove':
                            self.respond_client(self.execute_remove(command))

                        elif command['mode'] == 'switch':
                            self.respond_client(self.execute_switch(command))

                        elif command['mode'] == 'send':
                            self.respond_client(self.execute_send(command))

                        elif command['mode'] == 'status':
                            self.respond_client(self.execute_status(command))

                        elif command['mode'] == 'reset':
                            self.respond_client(self.execute_reset())

                        elif command['mode'] == 'start':
                            self.respond_client(self.execute_start())

                        elif command['mode'] == 'pause':
                            self.respond_client(self.execute_pause(command))

                        elif command['mode'] == 'restart':
                            self.respond_client(self.execute_restart(command))

                        elif command['mode'] == 'stop':
                            self.respond_client(self.execute_stop(command))

                        elif command['mode'] == 'kill':
                            self.respond_client(self.execute_kill(command))

                        elif command['mode'] == 'STOPDAEMON':
                            self.respond_client({'message': 'Pueue daemon shutting down',
                                                'status': 'success'})
                            # Kill current process and set active
                            # to False to stop while loop
                            self.active = False
                            self.execute_kill({'remove': False})
                            break

        self.socket.close()
        os.remove(get_socket_path())
        sys.exit(0)

    def read_queue(self):
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

    def write_queue(self):
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

    def execute_add(self, command):
        # Add command to queue and save it
        self.queue[self.nextKey] = command
        self.queue[self.nextKey]['status'] = 'queued'
        self.queue[self.nextKey]['returncode'] = ''
        self.queue[self.nextKey]['start'] = ''
        self.queue[self.nextKey]['end'] = ''
        self.nextKey += 1
        self.write_queue()
        return {'message': 'Command added', 'status': 'success'}

    def execute_remove(self, command):
        key = command['key']
        if key not in self.queue:
            # Send error answer to client in case there exists no such key
            answer = {'message': 'No command with key #{}'.format(str(key)), 'status': 'error'}
        else:
            # Delete command from queue, save the queue and send response to client
            if not self.paused and key == self.current_key:
                answer = {
                    'message': "Can't remove currently running process, "
                    "please stop the process before removing it.",
                    'status': 'error'
                }
            else:
                del self.queue[key]
                self.write_queue()
                answer = {'message': 'Command #{} removed'.format(key), 'status': 'success'}
        return answer

    def execute_restart(self, command):
        key = command['key']
        if key not in self.queue:
            # Send error answer to client in case there exists no such key
            answer = {'message': 'No command with key #{}'.format(str(key)), 'status': 'error'}
        else:
            # Delete command from queue, save the queue and send response to client
            if self.queue[key]['status'] == 'queued':
                answer = {'message': 'Command #{} is already queued'
                          .format(key), 'status': 'success'}
            if self.queue[key]['status'] in ['running', 'stopping', 'killing']:
                answer = {'message': 'Command #{} is currently running'
                          .format(key), 'status': 'error'}
            else:
                self.queue[self.nextKey] = {}
                self.queue[self.nextKey]['command'] = self.queue[key]['command']
                self.queue[self.nextKey]['path'] = self.queue[key]['path']
                self.queue[self.nextKey]['status'] = 'queued'
                self.queue[self.nextKey]['returncode'] = ''
                self.nextKey += 1
                self.write_queue()
                answer = {'message': 'Command #{} queued again'
                          .format(key), 'status': 'success'}
        return answer

    def execute_switch(self, command):
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
            if not self.paused and (first == self.current_key or second == self.current_key):
                answer = {
                    'message': "Can't switch currently running process, "
                    "please stop the process before switching it.",
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

    def execute_send(self, command):
        processInput = command['input']
        if self.process:
            self.process.stdin.write(processInput)
            self.process.stdin.flush()
            return {
                'message': 'Message sent',
                'status': 'success'
            }
        else:
            return {
                'message': 'No process running.',
                'status': 'failed'
            }

    def execute_status(self, command):
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
            data = deepcopy(self.queue)
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

    def execute_reset(self):
        # Terminate current process

        if self.process is not None:
            try:
                self.process.terminate(timout=10)
            except:
                self.process.kill()
            self.process.wait()

        self.reset = True
        self.log(rotate=True)

        answer = {'message': 'Reseting current queue', 'status': 'success'}
        return answer

    def execute_start(self):
        # Start the process if it is paused
        if self.process is not None and self.paused:
            os.kill(self.process.pid, signal.SIGCONT)
            self.queue[self.current_key]['status'] = 'running'
            self.processStatus = 'running'

        # Start the daemon if in paused state
        if self.paused:
            self.paused = False
            answer = {'message': 'Daemon started', 'status': 'success'}
        else:
            answer = {'message': 'Daemon already running', 'status': 'success'}
        return answer

    def execute_pause(self, command):
        # Pause the currently running process
        if self.process is not None and not self.paused and not command['wait']:
            os.kill(self.process.pid, signal.SIGSTOP)
            self.queue[self.current_key]['status'] = 'paused'
            self.processStatus = 'paused'

        # Pause the daemon
        if not self.paused:
            self.paused = True
            answer = {'message': 'Daemon paused', 'status': 'success'}
        else:
            answer = {'message': 'Daemon already paused', 'status': 'success'}
        return answer

    def execute_stop(self, command):
        if (self.process is not None):
            # Check if process just exited at this moment
            self.process.poll()
            if self.process.returncode is None:
                # Terminate process
                self.process.terminate()

                # Stop daemon
                self.stopping = True
                if command['remove']:
                    self.remove_current = True

                # Set status of current process in queue back to `queued`
                self.queue[self.current_key]['status'] = 'stopping'

                answer = {'message': 'Terminating current process and paused daemon',
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

    def execute_kill(self, command):
        if (self.process is not None):
            # Check if process just exited at this moment
            self.process.poll()
            if self.process.returncode is None:
                # Kill process
                self.process.kill()

                # Stop daemon
                self.stopping = True
                if command['remove']:
                    self.remove_current = True

                # Set status of current process in queue back to `queued`
                self.queue[self.current_key]['status'] = 'killing'

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
