import os
import sys
import stat
import getpass


def create_config_dir():
    home = os.path.expanduser('~')
    queueFolder = home+'/.config/pueue'
    if not os.path.exists(queueFolder):
        os.makedirs(queueFolder)
    return queueFolder


def create_log_dir():
    home = os.path.expanduser('~')
    logFolder = home+'/.local/share/pueue'
    if not os.path.exists(logFolder):
        os.makedirs(logFolder)
    return logFolder


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
