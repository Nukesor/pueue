import os


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
