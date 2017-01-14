import os
import pickle

from pueue.helper.socket import (
    connect_client_socket,
    receive_data,
    process_response,
)


def execute_add(args, root_dir=None):
    client = connect_client_socket(root_dir)

    # Send new instruction to daemon
    instruction = {
        'mode': 'add',
        'command': args['command'],
        'path': os.getcwd()
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_remove(args, root_dir=None):
    client = connect_client_socket(root_dir)

    # Send new instruction to daemon
    instruction = {
        'mode': 'remove',
        'key': args['key']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_restart(args, root_dir=None):
    client = connect_client_socket(root_dir)

    # Send new instruction to daemon
    instruction = {
        'mode': 'restart',
        'key': args['key']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_stop(args, root_dir=None):
    client = connect_client_socket(root_dir)

    # Send new instruction to daemon
    instruction = {
        'mode': 'stop',
        'remove': args['remove']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_pause(args, root_dir=None):
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


def execute_kill(args, root_dir=None):
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


def execute_switch(args, root_dir=None):
    client = connect_client_socket(root_dir)

    # Send new instruction to daemon
    instruction = {
        'mode': 'switch',
        'first': args['first'],
        'second': args['second']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_send(args, root_dir=None):
    client = connect_client_socket(root_dir)

    # Send new instruction to daemon
    instruction = {
        'mode': 'send',
        'input': args['input'],
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receive_data(client)
    process_response(response)
