import pickle

from helper import getClientSocket

def executeStart (args):
    client = getClientSocket()
    instruction = {'mode': 'START'}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)


def executePause (args):
    client = getClientSocket()
    instruction = {'mode': 'PAUSE'}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)


def executeStop (args):
    client = getClientSocket()
    instruction = {'mode': 'STOP'}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

def executeExit (args):
    client = getClientSocket()
    instruction = {'mode': 'EXIT'}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

