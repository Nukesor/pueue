import os
import pickle

from pueue.helper.socket import connect_client_socket, receive_data, process_response


def execute_add(args):
    client = connect_client_socket()

    # Send new instruction to daemon
    instruction = {
        'mode': 'add',
        'command': args['command'],
        'path': os.getcwd(),
        'status': 'queued',
        'returncode': ''
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receive_data(client)
    process_response(response)


def execute_remove(args):
    client = connect_client_socket()

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


def execute_restart(args):
    client = connect_client_socket()

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


def execute_stop(args):
    client = connect_client_socket()

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


def execute_kill(args):
    client = connect_client_socket()

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


def execute_switch(args):
    client = connect_client_socket()

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


def execute_send(args):
    client = connect_client_socket()

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
