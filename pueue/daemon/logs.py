import os
import time


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
            logFile.write('Command #{} exited with returncode {}:  "{}" \n'.format(key, log[key]['returncode'], log[key]['command']))
            logFile.write('Path: {} \n'.format(log[key]['path']))
            if log[key]['stderr']:
                logFile.write('Stderr output: \n    {}\n'.format(log[key]['stderr']))
            logFile.write('Stdout output: \n    {}'.format(log[key]['stdout']))
            logFile.write('\n')
        except:
            print('Errored while writing to log file. Wrong file permissions?')

    logFile.close()
