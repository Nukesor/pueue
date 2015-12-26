import os
import time

from colorclass import Color


def writeLog(logDir, log, rotate):
    # Get path  for logfile
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

    # Format and print Output
    for key in log:
        try:
            # Get returncode color:
            returncode = log[key]['returncode']
            if returncode == 0:
                returncode = Color('{autogreen}' + '{}'.format(returncode) + '{/autogreen}')
            else:
                returncode = Color('{autored}' + '{}'.format(returncode) + '{/autored}')

            # Command Id with returncode
            logFile.write(
                Color(
                    '{autoyellow}' +
                    'Command #{} '.format(key) +
                    '{/autoyellow}'
                ) +
                'exited with returncode {}: '.format(returncode) +
                '"{}" \n'.format(log[key]['command'])
            )
            logFile.write('Path: {} \n'.format(log[key]['path']))
            if log[key]['stderr']:
                logFile.write(
                    Color(
                        '{autored}' +
                        'Stderr output: ' +
                        '{/autored}'
                    ) +
                    ' \n    {}\n'.format(log[key]['stderr'])
                )
            if len(log[key]['stdout']) > 0:
                logFile.write(
                    Color(
                        '{autogreen}' +
                        'Stdout output: ' +
                        '{/autogreen}'
                    ) +
                    '\n    {}'.format(log[key]['stdout']))
            logFile.write('\n')
        except:
            print('Errored while writing to log file. Wrong file permissions?')

    logFile.close()
