import os
import sys
import stat
import getpass


def get_stdout_descriptor():
    userName = getpass.getuser()
    stdoutFile = '/tmp/pueueStdout' + userName
    if os.path.exists(stdoutFile):
        os.remove(stdoutFile)
    descriptor = open(stdoutFile, 'w+')

    # Set File permissionis for stdoutFile
    os.chmod(stdoutFile, stat.S_IRWXU)
    return descriptor


def get_stderr_descriptor():
    userName = getpass.getuser()
    stdoutFile = '/tmp/pueueStderr' + userName
    if os.path.exists(stdoutFile):
        os.remove(stdoutFile)
    descriptor = open(stdoutFile, 'w+')

    # Set File permissionis for stdoutFile
    os.chmod(stdoutFile, stat.S_IRWXU)
    return descriptor


def get_socket_path():
    # Generating pid and socket path from username
    try:
        userName = getpass.getuser()
    except:
        print("Couldn't get username from getpass.getuser(), aborting")
        sys.exit(1)
    else:
        socketPath = "/tmp/pueueSocket@"+userName+".sock"
        return socketPath
