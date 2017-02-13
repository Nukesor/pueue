import os
import time
import signal
import subprocess

from datetime import datetime

class ProcessHandler():
    """Manage underlying processes.

    This class is responsible for spawning, handling and supervising processes.
    The ProcessHandler is capable of running a pool of processes.
    """
    def __init__(self, queue, config_dir):
        """Initializes the process handler.

        Create member variables.
        """
        self.config_dir = config_dir
        self.queue = queue

        self.stopped = False
        self.max_processes = 1
        self.processes = {}
        self.descriptors = {}

        self.paused = []
        self.stopping = []
        self.to_remove = []

    def set_max(self, amount):
        """Set the amount of concurrent running processes."""
        self.max_processes = amount

    def is_running(self, key):
        """Return if there is a running process for this key."""
        return key in self.processes

    def all_finished(self):
        """Return `False`, if there are any active processes."""
        return not bool(len(self.processes))

    def wait_for_finish(self):
        """Wait until all processes finished."""
        while not self.all_finished():
            self.check_finished()
            time.sleep(0.5)

    def get_descriptor(self, number):
        """Create file descriptors for process output."""
        # Create stdout file and get file descriptor
        stdout_path = os.path.join(self.config_dir,
                                   'pueue_process_{}.stdout'.format(number))
        if os.path.exists(stdout_path): os.remove(stdout_path)
        out_descriptor = open(stdout_path, 'w+')

        # Create stderr file and get file descriptor
        stderr_path = os.path.join(self.config_dir,
                                   'pueue_process_{}.stderr'.format(number))
        if os.path.exists(stderr_path): os.remove(stderr_path)
        err_descriptor = open(stderr_path, 'w+')

        self.descriptors[number] = {}
        self.descriptors[number]['stdout'] =  out_descriptor
        self.descriptors[number]['stdout_path'] = stdout_path
        self.descriptors[number]['stderr'] =  err_descriptor
        self.descriptors[number]['stderr_path'] = stderr_path
        return out_descriptor, err_descriptor

    def clean_descriptor(self, number):
        """Close file descriptor and remove underlying files."""
        self.descriptors[number]['stdout'].close()
        self.descriptors[number]['stderr'].close()

        if os.path.exists(self.descriptors[number]['stdout_path']):
            os.remove(self.descriptors[number]['stdout_path'])

        if os.path.exists(self.descriptors[number]['stderr_path']):
            os.remove(self.descriptors[number]['stderr_path'])

    def check_finished(self):
        """Poll all processes and handle any finished processes.
        """
        changed = False
        for key in list(self.processes.keys()):
            # Poll process and check if it finshed
            process = self.processes[key]
            process.poll()
            if process.returncode is not None:
                # If a process is terminated by `stop` or `kill`
                # we want to queue it again instead closing it as failed.
                if key not in self.stopping:
                    # Get std_out and err_out
                    output, error_output = process.communicate()

                    descriptor = self.descriptors[key]
                    descriptor['stdout'].seek(0)
                    output = descriptor['stdout'].read().replace('\n', '\n    ')

                    descriptor['stderr'].seek(0)
                    error_output = descriptor['stderr'].read().replace('\n', '\n    ')

                    # Mark queue entry as finished and save returncode
                    self.queue[key]['returncode'] = process.returncode
                    if process.returncode != 0:
                        self.queue[key]['status'] = 'errored'
                    else:
                        self.queue[key]['status'] = 'done'

                    # Add outputs to queue
                    self.queue[key]['stdout'] = output
                    self.queue[key]['stderr'] = error_output
                    self.queue[key]['end'] = str(datetime.now().strftime("%H:%M"))

                    self.queue.write()
                    changed = True
                else:
                    self.stopping.remove(key)
                    if key in self.to_remove:
                        self.to_remove.remove(key)
                        del self.queue[key]
                    else:
                        self.queue[key]['status'] = 'queued'
                        self.queue[key]['start'] = ''
                        self.queue[key]['end'] = ''

                self.clean_descriptor(key)
                del self.processes[key]

        # If anything should be logged we return True
        return changed

    def check_for_new(self):
        free_slots = self.max_processes - len(self.processes)
        for item in range(free_slots):
            key = self.queue.next()
            if key is not None:
                self.spawn_new(key)

    def spawn_new(self, key):
        # Check if path exists
        if not os.path.exists(self.queue[key]['path']):
            self.queue[key]['status'] = 'errored'
            error_msg = "The directory for this command doesn't exist anymore: {}".format(self.queue[key]['path'])
            print(error_msg)
            self.queue[key]['stdout'] = ''
            self.queue[key]['stderr'] = error_msg

        else:
            # Get file descriptors
            stdout, stderr = self.get_descriptor(key)

            # Create subprocess
            self.processes[key] = subprocess.Popen(
                self.queue[key]['command'],
                shell=True,
                stdout=stdout,
                stderr=stderr,
                stdin=subprocess.PIPE,
                universal_newlines=True,
                cwd=self.queue[key]['path']
            )
            self.queue[key]['status'] = 'running'
            self.queue[key]['start'] = str(datetime.now().strftime("%H:%M"))

    def send_to_process(self, message, key):
        self.processes[key].stdin.write(message)
        self.processes[key].stdin.flush()
        return {
            'message': 'Message sent',
            'status': 'success'
        }

    def start_all(self):
        """Start all running processes."""
        for key in self.processes.keys():
            self.start_process(key)

    def pause_all(self):
        """Pause all running processes."""
        for key in self.processes.keys():
            self.pause_process(key)

    def stop_all(self):
        """Stop all running processes."""
        for key in self.processes.keys():
            self.stop_process(key)

    def kill_all(self):
        """Kill all running processes."""
        for key in self.processes.keys():
            self.stop_process(key, kill=True)

    def start_process(self, key):
        """Start a specific processes."""
        if key in self.processes and key in self.paused:
            os.kill(self.processes[key].pid, signal.SIGCONT)
            self.queue[key]['status'] = 'running'
            self.paused.remove(key)
            return True
        elif key not in self.processes:
            if self.queue[key]['status'] == 'queued':
                self.spawn_new(key)
                return True

        return False

    def pause_process(self, key):
        """Pause a specific processes."""
        if key in self.processes and key not in self.paused:
            os.kill(self.processes[key].pid, signal.SIGSTOP)
            self.queue[key]['status'] = 'paused'
            self.paused.append(key)
            return True
        return False

    def kill_process(self, key, remove=False):
        self.stop_process(key, remove, kill=True)

    def stop_process(self, key, remove=False, kill=False):
        if key in self.processes:
            self.processes[key].poll()
            if self.processes[key].returncode is None:
                # Kill process
                if kill:
                    self.processes[key].kill()
                    self.queue[key]['status'] = 'killing'
                else:
                    self.processes[key].terminate()
                    self.queue[key]['status'] = 'stopping'

                self.stopping.append(key)
                if remove:
                    self.to_remove.append(key)
            return True
        return False
