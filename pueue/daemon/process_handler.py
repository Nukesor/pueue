import os

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

        self.max_processes = 1
        self.processes = {}
        self.descriptors = {}

    def set_max(self, amount):
        self.max_processes = amount

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

    def clean_descriptor(self, number):
        """Close file descriptor and remove underlying files."""
        self.descriptor[number]['stdout'].close()
        self.descriptor[number]['stderr'].close()

        if os.path.exists(self.descriptors[number]['stdout_path']):
            os.remove(self.descriptors[number]['stdout_path'])

        if os.path.exists(self.descriptors[number]['stderr_path']):
            os.remove(self.descriptors[number]['stderr_path'])

    def check_finished(self):
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
                        del self.queue[self.queue.current_key]
                    else:
                        self.queue.current['status'] = 'queued'

                self.process = None
                self.processStatus = 'No running process'

    def spawn_new(self):
        if self.process is None:
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

    def send_to_process(self, data, key=None):
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

    def start_process(self, key=None):
        if self.process is not None and self.paused:
            os.kill(self.process.pid, signal.SIGCONT)
            self.queue.current['status'] = 'running'
            self.processStatus = 'running'

    def pause_process(self, key=None):
        # Pause the currently running process
        if self.process is not None and not self.paused and not payload['wait']:
            os.kill(self.process.pid, signal.SIGSTOP)
            self.queue.current['status'] = 'paused'
            self.processStatus = 'paused'

    def stop_process(self, key=None):
        if (self.process is not None):
            # Check if process just exited at this moment
            self.process.poll()
            if self.process.returncode is None:
                # Terminate process
                self.process.terminate()

    def kill_process(self, key=None):
        if (self.process is not None):
            # Check if process just exited at this moment
            self.process.poll()
            if self.process.returncode is None:
                # Kill process
                self.process.kill()
