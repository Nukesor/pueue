import os


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
