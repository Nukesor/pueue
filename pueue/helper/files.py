import os
import sys
import stat
import getpass


def createConfigDir():
    home = os.path.expanduser('~')
    queueFolder = home+'/.config/pueue'
    if not os.path.exists(queueFolder):
        os.makedirs(queueFolder)
    return queueFolder


def createLogDir():
    home = os.path.expanduser('~')
    logFolder = home+'/.local/share/pueue'
    if not os.path.exists(logFolder):
        os.makedirs(logFolder)
    return logFolder


def getStdoutDescriptor():
    userName = getpass.getuser()
    stdoutFile = '/tmp/pueueStdout' + userName
    if os.path.exists(stdoutFile):
        os.remove(stdoutFile)
    descriptor = open(stdoutFile, 'w+')

    # Set File permissionis for stdoutFile
    os.chmod(stdoutFile, stat.S_IRWXU)
    return descriptor


def getStderrDescriptor():
    userName = getpass.getuser()
    stdoutFile = '/tmp/pueueStderr' + userName
    if os.path.exists(stdoutFile):
        os.remove(stdoutFile)
    descriptor = open(stdoutFile, 'w+')

    # Set File permissionis for stdoutFile
    os.chmod(stdoutFile, stat.S_IRWXU)
    return descriptor


def getSocketPath():
    # Generating pid and socket path from username
    try:
        userName = getpass.getuser()
    except:
        print("Couldn't get username from getpass.getuser(), aborting")
        sys.exit(1)
    else:
        socketPath = "/tmp/pueueSocket@"+userName+".sock"
        return socketPath
