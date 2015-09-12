import os, pickle

def writeQueue(queue):
    home = os.path.expanduser('~')
    queueFolder = home+'/.pueue'
    queuePath = home+'/.pueue/queue'
    if not os.path.exists(queueFolder):
        os.makedirs(queueFolder)
    queueFile = open(queuePath,'wb+')
    try:
        pickle.dump(queue, queueFile, -1)
    except:
        print("Error while writing to queue file. Wrong file permissions?")
    queueFile.close()
