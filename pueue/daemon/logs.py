import os
import time

from colorclass import Color


def writeLog(logDir, log, rotate):
    # Get path for logfile
    if rotate:
        timestamp = time.strftime('%Y%m%d-%H%M')
        logPath = logDir + '/queue-' + timestamp + '.log'
    else:
        logPath = logDir + '/queue.log'

    # Remove existing Log
    if os.path.exists(logPath):
        os.remove(logPath)

    logFile = open(logPath, 'w')
    logFile.write('Pueue log for executed Commands: \n \n')

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

                # Command Id with returncode and actual command
                logFile.write(
                    Color('{autoyellow}' + 'Command #{} '.format(key) + '{/autoyellow}') +
                    'exited with returncode {}: '.format(returncode) +
                    '"{}" \n'.format(logentry['command'])
                )
                # Print Path
                logFile.write('Path: {} \n'.format(logentry['path']))

                # Print STDERR
                if logentry['stderr']:
                    logFile.write(Color('{autored}Stderr output: {/autored}\n    ') + logentry['stderr'])

                # Print STDOUT
                if len(logentry['stdout']) > 0:
                    logFile.write(Color('{autogreen}Stdout output: {/autogreen}\n    ') + logentry['stdout'])

                logFile.write('\n')
            except:
                print('Errored while writing to log file. Wrong file permissions?')

    logFile.close()
