import os, pickle

def readQueue():
    home = os.path.expanduser('~')
    queueFolder = home+'/.pueue'
    queuePath = home+'/.pueue/queue'
    if not os.path.exists(queueFolder):
        os.makedirs(queueFolder)
    if os.path.exists(queuePath):
        queueFile = open(queuePath,'rb')
        try:
            queue = pickle.load(queueFile)
        except:
            print("Queue file corrupted, deleting old queue")
            os.remove(queuePath)
            queue = {}
        queueFile.close()
    else:
        queue = {}
    return queue
