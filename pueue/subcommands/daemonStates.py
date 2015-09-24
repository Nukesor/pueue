import pickle

from pueue.helper.socket import getClientSocket


def daemonState(state):
    def changeState(args):
        client = getClientSocket()
        instruction = {'mode': state}
        data_string = pickle.dumps(instruction, -1)
        client.send(data_string)
        answer = client.recv(8192)
        print(pickle.loads(answer))
        client.close()
    return changeState
