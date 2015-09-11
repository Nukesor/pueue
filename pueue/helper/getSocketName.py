import sys, getpass

def getSocketName():
    # Generating pid and socket path from username
    try:
        userName = getpass.getuser()
    except:
        print("Couldn't get username from getpass.getuser(), aborting")
        sys.exit(1)
    else:
        socketPath = "/tmp/pueueSocket@"+userName
        return socketPath

