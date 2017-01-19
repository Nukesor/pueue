import os
import sys
import pickle
import select
import signal
import subprocess
import configparser
from copy import deepcopy
from datetime import datetime

from pueue.daemon.logs import write_log, remove_old_logs
from pueue.helper.socket import create_daemon_socket
from pueue.helper.files import cleanup
from pueue.daemon.queue import Queue
from pueue.daemon.process_handler import ProcessHandler


class Daemon():
    """The pueue daemon class.

    This is the central piece of code, which contains all client<->daemon
    communication code. The daemon manages the processes and the queue
    with the help of two other classes `ProcessHandler` and `Queue`.
    """
    def __init__(self, root_dir=None):
        """Initializes the daemon.

        Creates all needed directories, reads previous pueue sessions
        and the configuration files.
        """
        self.initialize_directories(root_dir)
        self.read_config()

        remove_old_logs(self.config['log']['logTime'], self.log_dir)
        # Initialize queue
        self.queue = Queue(self)
        self.process_handler = ProcessHandler(self.queue, self.config_dir)
        self.process_handler.set_max(self.config['default']['maxProcesses'])
        # Rotate logs, if all items from the last session finished
        if self.queue.next() is None:
            # Rotate old log
            self.log(rotate=True)
            # Remove old log file
            self.log()
            self.queue.write()
        self.socket = create_daemon_socket(self.config_dir)

        # If there are still jobs in the queue the daemon might pause,
        # if this behavior is defined in the config file.
        # The old log file is being loaded as well.
        self.paused = False
        if len(self.queue) > 0 and not self.config['default']['resumeAfterStart']:
            self.paused = True

        self.running = True
        self.stopping = False
        self.reset = False
        self.remove_current = False

        # Variables for handling sockets and child process
        self.clientAddress = None
        self.clientSocket = None
        self.process = None
        self.read_list = [self.socket]

    def initialize_directories(self, root_dir):
        """Create all directories needed for logs and configs."""
        if not root_dir:
            root_dir = os.path.expanduser('~')

        # Create config directory, if it doesn't exist
        self.config_dir = os.path.join(root_dir, '.config/pueue')
        if not os.path.exists(self.config_dir):
            os.makedirs(self.config_dir)

        # Create log directory, if it doesn't exist
        self.log_dir = os.path.join(root_dir, '.local/share/pueue')
        if not os.path.exists(self.log_dir):
            os.makedirs(self.log_dir)

    def respond_client(self, answer):
        """Generic function to send an answer to the client."""
        response = pickle.dumps(answer, -1)
        self.clientSocket.send(response)
        self.read_list.remove(self.clientSocket)
        self.clientSocket.close()

    def read_config(self):
        """Read a previous configuration file or create a new with default values."""
        config_file = os.path.join(self.config_dir, 'pueue.ini')
        self.config = configparser.ConfigParser()
        # Try to get configuration file and return it
        # If this doesn't work, a new default config file will be created
        if os.path.exists(config_file):
            try:
                self.config.read(config_file)
                return
            except:
                print('Error while parsing config file. Deleting old config')

        self.config['app'] = {
            'stopAtError': True,
            'resumeAfterStart': False,
            'maxProcesses': 1,
        }
        self.config['log'] = {
            'logTime': 60*60*24*14,
        }
        self.write_config()

    def write_config(self):
        """Write the current configuration to the config file."""
        config_file = os.path.join(self.config_dir, 'pueue.ini')
        with open(config_file, 'w') as file_descriptor:
            self.config.write(file_descriptor)

    def log(self, rotate=False):
        """Write process output and status to a log file."""
        write_log(self.log_dir, self.queue, rotate)

    def main(self):
        """The main function containing the loop for communication and process management.

        This function is the heart of the daemon.
        It is responsible for:
        - Client communication
        - Calling the ProcessHandler API.
        - Logging
        - Cleanup on exit

        """
        while self.running:
            # Check if there is a running process

            self.process_handler.check_finished()

            if self.reset:
                # Rotate log
                self.log(rotate=True)

                # Reset  queue
                self.queue.reset()

                # Reset Log
                self.log()
                self.reset = False

            # Start next Process
            if not self.paused:
                self.process_handler.spawn_new()

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
                        }

                        if payload['mode'] in functions.keys():
                            response = functions[payload['mode']](payload)
                            self.respond_client(response)
                        else:
                            self.respond_client({'message': 'Unknown Command',
                                                'status': 'error'})

        # Close socket, clean everything up and exit
        self.socket.close()
        cleanup()
        sys.exit(0)

    def stop_daemon(self, payload=None):
        """Kill current processes and initiate daemon shutdown."""
        self.process_handler.kill_all()
        self.running = False

        return {'message': 'Pueue daemon shutting down',
                'status': 'success'}

    def pipe_to_process(self, payload):
        """Send something to stdin of a specific process."""
        message = payload['input']
        key = payload['key']
        self.process_handler.send_to_process(message, key)

    def send_status(self, payload):
        """Send the daemon status and the current queue for displaying."""
        answer = {}
        data = []
        # Get daemon status
        if self.paused:
            answer['status'] = 'paused'
        else:
            answer['status'] = 'running'

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
        """Kill all processes, delete the queue and clean everything up."""

        self.process_handler.kill_all()
        self.reset = True

        answer = {'message': 'Resetting current queue', 'status': 'success'}
        return answer

    def start(self, payload):
        """Start the daemon and all processes or only a specific process."""
        # Start a specific process, if we have a key in our payload
        if 'key' in payload:
            success = self.process_handler.start_process(payload['key'])
            if success:
                answer = {'message': 'Process started.', 'status': 'success'}
            else:
                answer = {'message': 'No paused process with this key.',
                          'status': 'error'}

        # Start a all processes and the daemon
        else:
            if self.paused:
                self.paused = False
                self.process_handler.start_all()
                answer = {'message': 'Daemon and all processes started.',
                          'status': 'success'}
            else:
                answer = {'message': 'Daemon already running, starting all paused processes.',
                          'status': 'success'}
        return answer

    def pause(self, payload):
        """Start the daemon and all processes or only a specific process."""
        # Pause a specific process, if we have a key in our payload
        if 'key' in payload:
            success = self.process_handler.pause_process(payload['key'])
            if success:
                answer = {'message': 'Process paused.', 'status': 'success'}
            else:
                answer = {'message': 'No running process with this key.',
                          'status': 'error'}

        # Pause all processes and the daemon
        else:
            if not self.paused:
                self.paused = True
                self.process_handler.pause_all()
                answer = {'message': 'Daemon and all processes started.',
                          'status': 'success'}
            else:
                answer = {'message': 'Daemon already paused, pausing all processes anyway.',
                          'status': 'success'}
        return answer

    def stop_process(self, payload):
        """Pause the daemon and stop all processes or stop a specific process."""
        # Stop a specific process, if we have a key in our payload
        if 'key' in payload:
            success = self.process_handler.stop_process(payload['key'])
            if success:
                answer = {'message': 'Process stopping.', 'status': 'success'}
            else:
                answer = {'message': 'No running process with this key.',
                          'status': 'error'}

        # Stop all processes and the daemon
        else:
            if not self.paused:
                self.paused = True
                self.process_handler.stop_all()
                answer = {'message': 'Daemon paused and all processes stopped.',
                          'status': 'success'}
            else:
                answer = {'message': 'Daemon already paused, stopping all processes.',
                          'status': 'success'}
        return answer

    def kill_process(self, payload):
        """Pause the daemon and kill all processes or kill a specific process."""
        # Kill a specific process, if we have a key in our payload
        if 'key' in payload:
            success = self.process_handler.kill_process(payload['key'])
            if success:
                answer = {'message': 'Process killed.', 'status': 'success'}
            else:
                answer = {'message': 'No running process with this key.',
                          'status': 'error'}

        # Kill all processes and the daemon
        else:
            if not self.paused:
                self.paused = True
                self.process_handler.kill_all()
                answer = {'message': 'Daemon paused and all processes kill.',
                          'status': 'success'}
            else:
                answer = {'message': 'Daemon already paused, kill all processes.',
                          'status': 'success'}
        return answer
