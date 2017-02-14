import os
import sys
import time
import glob
import logging

from datetime import datetime
from logging.handlers import RotatingFileHandler

from colorclass import Color


class Logger():
    """The logger class which handles all kinds of daemon logging.

    This class handles two different things:

    1. Writing all logs of the finished processes (stdout, stderr, status, time, etc)
    2. Log rotation and deletion of named logs.
    3. Initialization of logging to file and stdout for daemon errors and information.
    """

    def __init__(self, root_dir):
        """Create log directory and initialize logger."""
        # Create log directory, if it doesn't exist
        self.log_dir = os.path.join(root_dir, '.local/share/pueue')
        if not os.path.exists(self.log_dir):
            os.makedirs(self.log_dir)

        # Initialize log file for daemon output
        self.daemon_log_path = os.path.join(self.log_dir, 'daemon.log')

        self.logger = logging.getLogger('')
        self.logger.setLevel(logging.INFO)
        log_format = logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")

        # Add a handler which outputs the log to stdout
        stdout_handler = logging.StreamHandler(sys.stdout)
        stdout_handler.setFormatter(log_format)
        self.logger.addHandler(stdout_handler)

        # Add a handler which outputs to a logfile
        file_handler = RotatingFileHandler(
            self.daemon_log_path,
            maxBytes=(1048576*5),
            backupCount=7
        )
        file_handler.setFormatter(log_format)
        self.logger.addHandler(file_handler)

    def debug(self, message):
        self.logger.debug(message)

    def info(self, message):
        self.logger.info(message)

    def warning(self, message):
        self.logger.warning(message)

    def error(self, message):
        self.logger.error(message)

    def exception(self, message=''):
        self.logger.exception(message)

    def rotate(self, log):
        """Move the current log to a new file with timestamp and create a new empty log file."""
        self.write(log, rotate=True)
        self.write({})

    def write(self, log, rotate=False):
        """Write the output of all finished processes to a compiled log file."""
        # Get path for logfile
        if rotate:
            timestamp = time.strftime('-%Y%m%d-%H%M')
            logPath = os.path.join(self.log_dir, 'queue{}.log'.format(timestamp))
        else:
            logPath = os.path.join(self.log_dir, 'queue.log')

        # Remove existing Log
        if os.path.exists(logPath):
            os.remove(logPath)

        log_file = open(logPath, 'w')
        log_file.write('Pueue log for executed Commands: \n \n')

        # Format, color and write log
        for key, logentry in log.items():
            if 'returncode' in logentry:
                try:
                    # Get returncode color:
                    returncode = logentry['returncode']
                    if returncode == 0:
                        returncode = Color('{autogreen}' + '{}'.format(returncode) + '{/autogreen}')
                    else:
                        returncode = Color('{autored}' + '{}'.format(returncode) + '{/autored}')

                    # Write command id with returncode and actual command
                    log_file.write(
                        Color('{autoyellow}' + 'Command #{} '.format(key) + '{/autoyellow}') +
                        'exited with returncode {}: '.format(returncode) +
                        '"{}" \n'.format(logentry['command'])
                    )
                    # Write path
                    log_file.write('Path: {} \n'.format(logentry['path']))
                    # Write times
                    log_file.write('Start: {}, End: {} \n'
                                   .format(logentry['start'], logentry['end']))

                    # Write STDERR
                    if logentry['stderr']:
                        log_file.write(Color('{autored}Stderr output: {/autored}\n    ') + logentry['stderr'])

                    # Write STDOUT
                    if len(logentry['stdout']) > 0:
                        log_file.write(Color('{autogreen}Stdout output: {/autogreen}\n    ') + logentry['stdout'])

                    log_file.write('\n')
                except Exception as a:
                    print('Failed while writing to log file. Wrong file permissions?')
                    print('Exception: {}'.format(str(a)))

        log_file.close()

    def remove_old(self, max_log_time):
        """Remove all logs which are older than the specified time."""
        files = glob.glob('{}/queue-*'.format(self.log_dir))
        files = list(map(lambda x: os.path.basename(x), files))

        for log_file in files:
            # Get time stamp from filename
            name = os.path.splitext(log_file)[0]
            timestamp = name.split('-', maxsplit=1)[1]

            # Get datetime from time stamp
            time = datetime.strptime(timestamp, '%Y%m%d-%H%M')
            now = datetime.now()

            # Get total delta in seconds
            delta = now - time
            seconds = delta.total_seconds()

            # Delete log file, if the delta is bigger than the specified log time
            if seconds > int(max_log_time):
                log_filePath = os.path.join(self.log_dir, log_file)
                os.remove(log_filePath)
