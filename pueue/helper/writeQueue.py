import os, pickle

def writeQueue(queue):
    queueFile = open(queuePath,'wb+')
    try:
        pickle.dump(queue, queueFile, -1)
    except:
        print("Error while writing to queue file. Wrong file permissions?")
    queueFile.close()
