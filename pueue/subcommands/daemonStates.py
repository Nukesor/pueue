import pickle

from helper import getClientSocket

def executeStart (args):
    client = getClientSocket()
    instruction = {'mode': 'START'}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)
    answer = client.recv(8192)
    print(pickle.loads(answer))
    client.close()


def executePause (args):
    client = getClientSocket()
    instruction = {'mode': 'PAUSE'}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)
    answer = client.recv(8192)
    print(pickle.loads(answer))
    client.close()


def executeStop (args):
    client = getClientSocket()
    instruction = {'mode': 'STOP'}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)
    answer = client.recv(8192)
    print(pickle.loads(answer))
    client.close()

def executeExit (args):
    client = getClientSocket()
    instruction = {'mode': 'EXIT'}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)
    answer = client.recv(8192)
    print(pickle.loads(answer))
    client.close()


def executeKill (args):
    client = getClientSocket()
    instruction = {'mode': 'KILL'}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)
    answer = client.recv(8192)
    print(pickle.loads(answer))
    client.close()


