import sys, getpass, os

def getSocketName():
    # Generating pid and socket path from username
    try:
        userName = getpass.getuser()
    except:
        print("Couldn't get username from getpass.getuser(), aborting")
        sys.exit(1)
    else:
        home = os.path.expanduser('~')
        queueFolder = home+'/.pueue'
        socketPath = queueFolder+"/pueueSocket@"+userName+".sock"
        return socketPath

