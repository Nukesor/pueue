import pickle

from pueue.helper.socket import connect_client_socket, receive_data, process_response


# Factory function for simple command sending functions
def command_factory(command):
    def communicate(body={}):
        # Initialize socket, message and send it
        client = connect_client_socket()
        body['mode'] = command
        if 'func' in body:
            del body['func']
        data_string = pickle.dumps(body, -1)
        client.send(data_string)

        # Receive message and return it
        response = receive_data(client)
        return response
    return communicate


# Factory function for simple command sending functions
def print_command_factory(command):
    def communicate(body={}):
        # Initialize socket, message and send it
        client = connect_client_socket()
        body['mode'] = command
        body['func'] = None
        data_string = pickle.dumps(body, -1)
        client.send(data_string)

        # Receive message and print it. Exit with 1, if an error has been sent.
        response = receive_data(client)
        process_response(response)
    return communicate
