import os
import pickle

from pueue.client.socket import (
    connect_socket,
    receive_data,
    process_response,
)


def execute_add(args, root_dir=None):
    """Add a new command to the daemon queue.

    Args:
        args['command'] (list(str)): The actual programm call. Something like ['ls', '-a'] or ['ls -al']
        root_dir (string): The path to the root directory the daemon is running in.
    """
    client = connect_socket(root_dir)

    # We accept a list of strings.
    # This is done to create a better commandline experience with argparse.
    command = ' '.join(args['command'])

    # Send new instruction to daemon
    instruction = {
        'mode': 'add',
        'command': command,
        'path': os.getcwd()
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive answer from daemon and print it
    response = receive_data(client)
    process_response(response)
