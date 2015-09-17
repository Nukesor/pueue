import os
import pickle

from helper import getClientSocket

def executeAdd(args):
    client = getClientSocket()

    # Send new instruction to daemon
    instruction = {'mode': 'add', 'command': args.command, 'path': os.getcwd()}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    answer = client.recv(8192)
    print(pickle.loads(answer))
    client.close()

def executeRemove(args):
    client = getClientSocket()

    # Send new instruction to daemon
    instruction = {'mode': 'remove', 'key': args.key}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    answer = client.recv(8192)
    print(pickle.loads(answer))
    client.close()

