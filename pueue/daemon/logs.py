import os
import time
from datetime import datetime

from colorclass import Color


def write_log(log_dir, log, rotate):
    # Get path for logfile
    if rotate:
        timestamp = time.strftime('-%Y%m%d-%H%M')
        logPath = os.path.join(log_dir, 'queue{}.log'.format(timestamp))
    else:
        logPath = os.path.join(log_dir, 'queue.log')

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
                print('Errored while writing to log file. Wrong file permissions?')
                print('Exception: {}'.format(str(a)))

    log_file.close()


def remove_old_logs(log_time, log_dir):
    files = os.listdir(log_dir)

    for log_file in files:
        if log_file != 'queue.log':
            # Get time stamp from filename
            name = os.path.splitext(log_file)[0]
            timestamp = name.split('-', maxsplit=1)[1]

            # Get datetime from time stamp
            time = datetime.strptime(timestamp, '%Y%m%d-%H%M')
            now = datetime.now()

            # Get total delta in seconds
            delta = now - time
            seconds = delta.total_seconds()

            # Delete log file, if the timestamp is older than the specified log time
            if seconds > int(log_time):
                log_filePath = os.path.join(log_dir, log_file)
                os.remove(log_filePath)
