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
from pueue.helper.files import get_stdout_descriptor, get_stderr_descriptor
from pueue.daemon.queue import Queue


class Daemon():
    def __init__(self, root_dir=None):
        """Initializes the daemon.

        Creates all needed directories, reads previous pueue sessions
        and the configuration files.
        """
        self.initialize_directories(root_dir)
        self.config = get_config(self.config_dir)

        remove_old_logs(self.config['log']['logTime'], self.log_dir)
        # Initialize queue
        self.queue = Queue(self)
        # Rotate logs, if all items from the last session finished
        if self.queue.next() is None:
            # Rotate old log
            self.log(rotate=True)
            # Remove old log file
            self.log()
            self.queue.write()
        self.socket = create_daemon_socket()

        # If there are still jobs in the queue the daemon might pause,
        # if this behaviour is defined in the config file.
        # The old logfile is beeing loaded as well.
        self.paused = False
        if len(self.queue) > 0 and not self.config['default']['resumeAfterStart']:
            self.paused = True

        self.running = True
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

    def initialize_directories(self, root_dir):
        """Create all directories needed for logs and configs."""
        root_dir = os.path.expanduser('~')

        # Create config dir, if not existing
        self.config_dir = root_dir + '/.config/pueue'
        if not os.path.exists(self.config_dir):
            os.makedirs(self.config_dir)

        self.log_dir = root_dir + '/.local/share/pueue'
        if not os.path.exists(self.log_dir):
            os.makedirs(self.log_dir)

    def respond_client(self, answer):
        """Generic function to send an answer to the client."""
        response = pickle.dumps(answer, -1)
        self.clientSocket.send(response)
        self.read_list.remove(self.clientSocket)
        self.clientSocket.close()

    def log(self, rotate=False):
        # If there is a finished process a
        # human readable log will be written
        write_log(self.log_dir, self.queue, rotate)

    def main(self):
        while self.running:
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
                        self.queue.current['returncode'] = self.process.returncode
                        if self.process.returncode != 0:
                            self.queue.current['status'] = 'errored'
                        else:
                            self.queue.current['status'] = 'done'

                        # Add outputs to queue
                        self.queue.current['stdout'] = output
                        self.queue.current['stderr'] = error_output
                        self.queue.current['end'] = str(datetime.now().strftime("%H:%M"))

                        # Pause Daemon, if it is configured to stop
                        if self.config['default']['stopAtError'] is True and not self.reset:
                            if self.process.returncode != 0:
                                self.paused = True

                        self.queue.write()
                        self.log()
                    else:
                        # Process finally finished.
                        # Now we can set the status to paused.
                        self.paused = True
                        self.stopping = False
                        if self.remove_current:
                            self.remove_current = False
                            del self.queue.current
                        else:
                            self.queue.current['status'] = 'queued'

                    self.process = None
                    self.current_key = None
                    self.processStatus = 'No running process'

            if self.reset:
                # Rotate log
                self.log(rotate=True)

                # Reset  queue
                self.queue.reset()

                # Reset Log
                self.log()
                self.nextKey = 0
                self.reset = False

            # Start next Process
            if not self.paused and self.process is None:
                key = self.queue.next()
                if key is not None:
                    # Check if path exists
                    if not os.path.exists(self.queue.current['path']):
                        self.queue.current['status'] = 'errored'
                        error_msg = "The directory for this command doesn't exist any longer"
                        print(error_msg)
                        self.queue.current['stdout'] = ''
                        self.queue.current['stderr'] = error_msg

                    else:
                        # Remove the output from all stdout and stderr files
                        self.stdout.seek(0)
                        self.stdout.truncate()
                        self.stderr.seek(0)
                        self.stderr.truncate()
                        # Spawn subprocess
                        self.process = subprocess.Popen(
                            self.queue.current['command'],
                            shell=True,
                            stdout=self.stdout,
                            stderr=self.stderr,
                            stdin=subprocess.PIPE,
                            universal_newlines=True,
                            cwd=self.queue.current['path']
                        )
                        self.queue.current['status'] = 'running'
                        self.queue.current['start'] = str(datetime.now().strftime("%H:%M"))
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
                        instruction = None

                    # Check for valid instruction
                    if instruction is not None:
                        # Check if received data can be unpickled.
                        try:
                            payload = pickle.loads(instruction)
                        except EOFError:
                            # Instruction is ignored if it can't be unpickled
                            print('Received message is incomplete, dropping received data.')
                            self.read_list.remove(self.clientSocket)
                            self.clientSocket.close()
                            # Set invalid payload
                            payload = {'mode': ''}

                        functions = {
                            'add': self.queue.add_new,
                            'remove': self.queue.remove,
                            'switch': self.queue.switch,
                            'send': self.pipe_to_process,
                            'status': self.send_status,
                            'start': self.start,
                            'pause': self.pause,
                            'restart': self.queue.restart,
                            'stop': self.stop_process,
                            'kill': self.kill_process,
                            'reset': self.reset_everything,
                            'STOPDAEMON': self.stop_daemon,
                            'get_log_dir': self.send_log_dir,
                        }

                        if payload['mode'] in functions.keys():
                            response = functions[payload['mode']](payload)
                            self.respond_client(response)
                        else:
                            self.respond_client({'message': 'Unknown Command',
                                                'status': 'error'})

        self.socket.close()
        os.remove(get_socket_path())
        sys.exit(0)

    def stop_daemon(self, payload=None):
        self.respond_client({'message': 'Pueue daemon shutting down',
                            'status': 'success'})
        # Kill current process and set active
        # to False to stop while loop
        self.running = False
        self.kill_process({'remove': False})

    def pipe_to_process(self, payload):
        processInput = payload['input']
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

    def send_log_dir(self, payload):
        answer = {'log_dir': self.log_dir}
        return answer

    def send_status(self, payload):
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
            data = deepcopy(self.queue.queue)
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

    def reset_everything(self, payload):
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

    def start(self, payload):
        # Start the process if it is paused
        if self.process is not None and self.paused:
            os.kill(self.process.pid, signal.SIGCONT)
            self.queue.current['status'] = 'running'
            self.processStatus = 'running'

        # Start the daemon if in paused state
        if self.paused:
            self.paused = False
            answer = {'message': 'Daemon started', 'status': 'success'}
        else:
            answer = {'message': 'Daemon already running', 'status': 'success'}
        return answer

    def pause(self, payload):
        # Pause the currently running process
        if self.process is not None and not self.paused and not payload['wait']:
            os.kill(self.process.pid, signal.SIGSTOP)
            self.queue.current['status'] = 'paused'
            self.processStatus = 'paused'

        # Pause the daemon
        if not self.paused:
            self.paused = True
            answer = {'message': 'Daemon paused', 'status': 'success'}
        else:
            answer = {'message': 'Daemon already paused', 'status': 'success'}
        return answer

    def stop_process(self, payload):
        if (self.process is not None):
            # Check if process just exited at this moment
            self.process.poll()
            if self.process.returncode is None:
                # Terminate process
                self.process.terminate()

                # Stop daemon
                self.stopping = True
                if payload['remove']:
                    self.remove_current = True

                # Set status of current process in queue back to `queued`
                self.queue.current['status'] = 'stopping'

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

    def kill_process(self, payload):
        if (self.process is not None):
            # Check if process just exited at this moment
            self.process.poll()
            if self.process.returncode is None:
                # Kill process
                self.process.kill()

                # Stop daemon
                self.stopping = True
                if payload['remove']:
                    self.remove_current = True

                # Set status of current process in queue back to `queued`
                self.queue.current['status'] = 'killing'

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
