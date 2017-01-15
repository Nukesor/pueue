import os
import sys
import stat
import getpass


def get_file_descriptor(config_dir):
    """Returns file descriptors for stderr and stdout of pueue's subprocess."""

    # Create stdout file and get file descriptor
    stdout_path = os.path.join(config_dir, 'pueue.stdout')
    if os.path.exists(stdout_path):
        os.remove(stdout_path)
    out_descriptor = open(stdout_path, 'w+')

    # Create stderr file and get file descriptor
    stderr_path = os.path.join(config_dir, 'pueue.stderr')
    if os.path.exists(stderr_path):
        os.remove(stderr_path)
    err_descriptor = open(stderr_path, 'w+')

    # Set File permissionis for files
    os.chmod(stdout_path, stat.S_IRWXU)
    os.chmod(stderr_path, stat.S_IRWXU)
    return out_descriptor, err_descriptor


def cleanup(config_dir):
    """Removes temporary stderr and stdout files as well as the daemon socket."""

    stdout_path = os.path.join(config_dir, 'pueue.stdout')
    stderr_path = os.path.join(config_dir, 'pueue.stderr')
    if os._exists(stdout_path):
        os.remove(stdout_path)
    if os._exists(stderr_path):
        os.remove(stderr_path)

    socketPath = os.path.join(config_dir, 'pueue.sock')
    if os.path.exists(socketPath):
        os.remove(socketPath)
