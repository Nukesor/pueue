import pickle
from textwrap import wrap
from terminaltables import AsciiTable

from pueue.helper.files import createDir
from pueue.helper.socket import getClientSocket


def executeShow(args):
    client = getClientSocket()
#    if hasattr(args, 'index') and args.index is not None:
#        instruction = {'mode': 'show', 'index': args.index}
#    else:
    instruction = {'mode': 'show', 'index': 'all'}

    # Send new instruction to daemon
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = client.recv(8192)
    answer = pickle.loads(response)
    print('Daemon: {}, Process status: {}, Returncode: {} \n'.format(answer['status'], answer['process'], answer['current']))
    data = answer['data']
    client.close()
    if isinstance(data, str):
        print(data)
    elif isinstance(data, dict):
        # Format incomming data to be compatible with Terminaltables
        formatted_data = []
        formatted_data.append(['Index', 'Command', 'Path'])
        for key, entry in data.items():
            formatted_data.append(['#{}'.format(key), entry['command'], entry['path']])

        # Create AsciiTable instance and define style
        table = AsciiTable(formatted_data)
        table.outer_border = False
        table.inner_column_border = False

        # Format long strings to match the console width
        max_width = table.column_max_width(1)
        for i, entry in enumerate(table.table_data):
            for j, string in enumerate(entry):
                wrapped_string = '\n'.join(wrap(string, max_width))
                table.table_data[i][j] = wrapped_string

        print(table.table)
    print('')


def executeLog(args):
    logPath = createDir() + '/log/queue.log'
    logFile = open(logPath, 'r')
    print(logFile.read())
