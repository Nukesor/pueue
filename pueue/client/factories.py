import pickle

from pueue.helper.socket import connect_client_socket, receive_data, process_response


# Factory function for simple command sending functions
def command_factory(state):
    def change_state(instruction):
        # Initialize socket, message and send it
        client = connect_client_socket()
        instruction['mode'] = state
        instruction['func'] = None
        data_string = pickle.dumps(instruction, -1)
        client.send(data_string)

        # Receive message and print it
        response = receive_data(client)
        process_response(response)
    return change_state
