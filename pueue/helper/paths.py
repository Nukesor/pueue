import os


def createDir():
    home = os.path.expanduser('~')
    queueFolder = home+'/.pueue'
    if not os.path.exists(queueFolder):
        os.makedirs(queueFolder)
    return queueFolder
