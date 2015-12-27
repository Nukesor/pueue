import pickle

from pueue.helper.socket import connectClientSocket, receiveData, processResponse


# Factory function for simple command sending functions
def commandFactory(state):
    def changeState(instruction):
        # Initialize socket, message and send it
        client = connectClientSocket()
        instruction['mode'] = state
        instruction['func'] = None
        data_string = pickle.dumps(instruction, -1)
        client.send(data_string)

        # Receive message and print it
        response = receiveData(client)
        processResponse(response)
    return changeState
