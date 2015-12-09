import os
import pickle

from pueue.helper.socket import getClientSocket, printResponse


def executeAdd(args):
    client = getClientSocket()

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
    printResponse(client)


def executeRemove(args):
    client = getClientSocket()

    # Send new instruction to daemon
    instruction = {
        'mode': 'remove',
        'key': args['key']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    printResponse(client)


def executeSwitch(args):
    client = getClientSocket()

    # Send new instruction to daemon
    instruction = {
        'mode': 'switch',
        'first': args['first'],
        'second': args['second']
    }
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    printResponse(client)
