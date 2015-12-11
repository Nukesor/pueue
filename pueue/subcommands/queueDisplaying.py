import time
import math
import pickle
import curses
import getpass

from textwrap import wrap
from functools import reduce

from pueue.helper.files import createLogDir
from pueue.helper.socket import getClientSocket

from terminaltables.tables import AsciiTable
from terminaltables.terminal_io import terminal_size


def executeStatus(args):
    client = getClientSocket()
    instruction = {'mode': 'status'}

    # Send new instruction to daemon
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    response = client.recv(8192)
    answer = pickle.loads(response)
    client.close()
    # First row, showing daemon status
    print('Daemon: {}, Process status: {}, Returncode: {} \n'.format(answer['status'], answer['process'], answer['current']))

    # Handle queue data
    data = answer['data']
    if isinstance(data, str):
        print(data)
    elif isinstance(data, dict):
        # Format incomming data to be compatible with Terminaltables
        formatted_data = []
        formatted_data.append(['Index', 'Status', 'Code', 'Command', 'Path'])
        for key, entry in data.items():
            formatted_data.append(
                [
                    '#{}'.format(key),
                    entry['status'],
                    '{}'.format(entry['returncode']),
                    entry['command'],
                    entry['path']
                ]
            )

        # Create AsciiTable instance and define style
        table = AsciiTable(formatted_data)
        table.outer_border = False
        table.inner_column_border = False

        terminal_width = terminal_size()
        customWidth = table.column_widths
        # If the text is wider than the actual terminal size, we
        # compute a new size for the Command and Path column.
        if (reduce(lambda a, b: a+b, table.column_widths) + 10) > terminal_width[0]:
            # We have to subtract 10 because of table paddings
            left_space = math.floor((terminal_width[0] - customWidth[0] - customWidth[1] - customWidth[2] - 12)/2)

            if customWidth[3] < left_space:
                customWidth[4] = 2*left_space - customWidth[3]
            elif customWidth[4] < left_space:
                customWidth[3] = 2*left_space - customWidth[4]
            else:
                customWidth[3] = left_space
                customWidth[4] = left_space

        # Format long strings to match the console width
        for i, entry in enumerate(table.table_data):
            for j, string in enumerate(entry):
                max_width = customWidth[j]
                wrapped_string = '\n'.join(wrap(string, max_width))
                table.table_data[i][j] = wrapped_string

        print(table.table)
    print('')


def executeLog(args):
    logPath = createLogDir() + '/queue.log'
    logFile = open(logPath, 'r')
    print(logFile.read())


def executeShow(args):
    # Get current pueueSTDout file from tmp
    userName = getpass.getuser()
    stdoutFile = '/tmp/pueueStdout{}'.format(userName)
    descriptor = open(stdoutFile, 'r')
    running = True
    # Continually print output with curses or just print once
    if args['watch']:
        stdscr = curses.initscr()
        while running:
            stdscr.clear()
            descriptor.seek(0)
            message = descriptor.read()
            stdscr.addstr(0, 0, message)
            stdscr.refresh()
            time.sleep(2)
    else:
        descriptor.seek(0)
        print(descriptor.read())
