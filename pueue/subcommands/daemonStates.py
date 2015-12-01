import pickle

from pueue.helper.socket import getClientSocket, printResponse


# Factory function for simple command sending functions
def daemonState(state):
    def changeState(args):
        # Initialize socket, message and send it
        client = getClientSocket()
        instruction = {'mode': state}
        data_string = pickle.dumps(instruction, -1)
        client.send(data_string)

        # Receive message and print it
        printResponse(client)
    return changeState
