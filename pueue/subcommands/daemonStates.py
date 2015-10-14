import sys
import pickle

from pueue.helper.socket import getClientSocket


def daemonState(state):
    def changeState(args):
        client = getClientSocket()
        instruction = {'mode': state}
        data_string = pickle.dumps(instruction, -1)
        client.send(data_string)
        answer = client.recv(8192)
        response = pickle.loads(answer)
        print(response['message'])
        client.close()
        if response['status'] != 'success':
            sys.exit(1)
    return changeState
