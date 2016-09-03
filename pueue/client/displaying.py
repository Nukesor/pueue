import time
import math
import pickle
import curses
import getpass

from textwrap import wrap
from functools import reduce
from colorclass import Color

from pueue.helper.files import createLogDir
from pueue.helper.socket import connectClientSocket

from terminaltables import AsciiTable
from terminaltables.terminal_io import terminal_size


def executeStatus(args):
    client = connectClientSocket()
    instruction = {'mode': 'status'}

    # Send new instruction to daemon
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    # Receive Answer from daemon and print it
    # About 1 MB buffersize for large queues with large paths
    response = client.recv(1048576)
    answer = pickle.loads(response)
    client.close()
    # First rows, showing daemon status
    if answer['status'] == 'running':
        answer['status'] = Color('{autogreen}' + '{}'.format(answer['status']) + '{/autogreen}')
    elif answer['status'] == 'paused':
        answer['status'] = Color('{autoyellow}' + '{}'.format(answer['status']) + '{/autoyellow}')

    if answer['process'] == 'running' or answer['process'] == 'paused':
        answer['process'] = Color('{autogreen}' + '{}'.format(answer['process']) + '{/autogreen}')

    print('Daemon: {}\nProcess status: {} \n'.format(answer['status'], answer['process']))

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
                if j == 1:
                    if wrapped_string == 'done' or wrapped_string == 'running' or wrapped_string == 'paused':
                        wrapped_string = Color('{autogreen}' + '{}'.format(wrapped_string) + '{/autogreen}')
                    elif wrapped_string == 'queued':
                        wrapped_string = Color('{autoyellow}' + '{}'.format(wrapped_string) + '{/autoyellow}')
                    elif wrapped_string == 'errored':
                        wrapped_string = Color('{autored}' + '{}'.format(wrapped_string) + '{/autored}')
                elif j == 2:
                    if wrapped_string == '0' and wrapped_string != 'Code':
                        wrapped_string = Color('{autogreen}' + '{}'.format(wrapped_string) + '{/autogreen}')
                    elif wrapped_string != '0' and wrapped_string != 'Code':
                        wrapped_string = Color('{autored}' + '{}'.format(wrapped_string) + '{/autored}')

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
        # Initialize curses
        stdscr = curses.initscr()
        curses.noecho()
        curses.cbreak()
        curses.curs_set(2)
        stdscr.keypad(True)
        stdscr.refresh()

        # Update output every two seconds
        while running:
            stdscr.clear()
            descriptor.seek(0)
            message = descriptor.read()
            stdscr.addstr(0, 0, message)
            stdscr.refresh()
            time.sleep(2)

        # Curses cleanup
        curses.nocbreak()
        stdscr.keypad(False)
        curses.echo()
        curses.endwin()
    else:
        descriptor.seek(0)
        print(descriptor.read())
