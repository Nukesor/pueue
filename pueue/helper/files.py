import os
import getpass

def createDir():
    home = os.path.expanduser('~')
    queueFolder = home+'/.pueue'
    if not os.path.exists(queueFolder):
        os.makedirs(queueFolder)
    return queueFolder

def createLogDir():
    home = os.path.expanduser('~')
    logFolder = home+'/.pueue/log'
    if not os.path.exists(logFolder):
        os.makedirs(logFolder)
    return logFolder

def getStdoutDescriptor():
    userName = getpass.getuser()
    stdoutFile = '/tmp/pueueStdout{}'.format(userName)
    if os.path.exists(stdoutFile):
        os.remove(stdoutFile)
    descriptor = open(stdoutFile, 'w+')
    return descriptor

