import os
import pickle

from pueue.helper.socket import (
    connect_client_socket,
    receive_data,
    process_response,
)


def execute_add(args, root_dir=None):
    """Add a new command to the daemon queue.

    Args:
        args['command'] (str): The actual programm call. Something like `ls -a`
        root_dir (string): The path to the root directory the daemon is running in.
    """
    client = connect_client_socket(root_dir)

    # Send new instruction to daemon
    instruction = {
        'mode': 'add',
        'command': args['command'],
        'path': os.getcwd()
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive answer from daemon and print it
    response = receive_data(client)
    process_response(response)
