import os
import pickle

from helper import getClientSocket

def executeShow(args):
    client = getClientSocket()
    if args.command:
        # Send new instruction to daemon
        addCommand = {'mode': 'add', 'command': args.command, 'path': os.getcwd()}
        data_string = pickle.dumps(addCommand, -1)
        client.send(data_string)
        # Receive Answer from daemon and print it
        answer = client.recv(8192)
        print(pickle.loads(answer))
        client.close()

