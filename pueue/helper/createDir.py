import os

def createDir():
    home = os.path.expanduser('~')
    queueFolder = home+'/.pueue'
    queuePath = home+'/.pueue/queue'
    if not os.path.exists(queueFolder):
        os.makedirs(queueFolder)
