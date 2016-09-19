import os
import pickle

from pueue.helper.socket import connectClientSocket, receiveData, processResponse


def executeAdd(args):
    client = connectClientSocket()

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
    response = receiveData(client)
    processResponse(response)


def executeRemove(args):
    client = connectClientSocket()

    # Send new instruction to daemon
    instruction = {
        'mode': 'remove',
        'key': args['key']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receiveData(client)
    processResponse(response)


def executeRestart(args):
    client = connectClientSocket()

    # Send new instruction to daemon
    instruction = {
        'mode': 'restart',
        'key': args['key']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receiveData(client)
    processResponse(response)


def executeStop(args):
    client = connectClientSocket()

    # Send new instruction to daemon
    instruction = {
        'mode': 'stop',
        'remove': args['remove']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receiveData(client)
    processResponse(response)


def executeKill(args):
    client = connectClientSocket()

    # Send new instruction to daemon
    instruction = {
        'mode': 'kill',
        'remove': args['remove']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receiveData(client)
    processResponse(response)


def executeSwitch(args):
    client = connectClientSocket()

    # Send new instruction to daemon
    instruction = {
        'mode': 'switch',
        'first': args['first'],
        'second': args['second']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receiveData(client)
    processResponse(response)


def executeSend(args):
    client = connectClientSocket()

    # Send new instruction to daemon
    instruction = {
        'mode': 'send',
        'input': args['input'],
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = receiveData(client)
    processResponse(response)
