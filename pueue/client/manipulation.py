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


def execute_remove(args, root_dir=None):
    """Remove a new command from the daemon queue.

    Args:
        args['key'] (int): The queue index of the programm call.
        root_dir (string): The path to the root directory the daemon is running in.
    """
    client = connect_client_socket(root_dir)

    instruction = {
        'mode': 'remove',
        'key': args['key']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_switch(args, root_dir=None):
    """Switch two commands in the daemon queue.

    Args:
        args['first'] (int): First command.
        args['second'] (int): Second command.
        root_dir (string): The path to the root directory the daemon is running in.
    """

    client = connect_client_socket(root_dir)

    instruction = {
        'mode': 'switch',
        'first': args['first'],
        'second': args['second']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_restart(args, root_dir=None):
    """Restart a failed or finished command in the daemon queue.

    Args:
        args['key'] (int): The queue index of the programm call.
        root_dir (string): The path to the root directory the daemon is running in.
    """
    client = connect_client_socket(root_dir)

    instruction = {
        'mode': 'restart',
        'key': args['key']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_pause(args, root_dir=None):
    """Pause the daemon and/or the underlying process (SIGSTOP).

    Args:
        args['wait'] (bool): If `True`, the underlying process won't be paused and the daemon
                            waits for it to finish.
        root_dir (string): The path to the root directory the daemon is running in.
    """
    client = connect_client_socket(root_dir)

    # Send new instruction to daemon
    instruction = {
        'mode': 'pause',
        'wait': args['wait']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_stop(args, root_dir=None):
    """Stop (SIGTERM) the current running command.

    Args:
        args['remove'] (bool): If `True`, the daemon will remove the
                              current running command from the queue entirely.
        root_dir (string): The path to the root directory the daemon is running in.
    """
    client = connect_client_socket(root_dir)

    instruction = {
        'mode': 'stop',
        'remove': args['remove']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_kill(args, root_dir=None):
    """Kill (SIGKILL) the current running command.

    Args:
        args['remove'] (bool): If `True`, the daemon will remove the
                              current running command from the queue entirely.
        root_dir (string): The path to the root directory the daemon is running in.
    """
    client = connect_client_socket(root_dir)

    # Send new instruction to daemon
    instruction = {
        'mode': 'kill',
        'remove': args['remove']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_send(args, root_dir=None):
    """Send something to the STDIN of the currently running process.

    Args:
        args['input'] (str): The input that will be sent to STDIN.
        root_dir (string): The path to the root directory the daemon is running in.
    """
    client = connect_client_socket(root_dir)

    instruction = {
        'mode': 'send',
        'input': args['input'],
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive answer from daemon and print it
    response = receive_data(client)
    process_response(response)
