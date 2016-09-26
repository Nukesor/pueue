import time
import math
import pickle
import curses
import getpass
import operator

from textwrap import wrap
from functools import reduce
from colorclass import Color

from pueue.helper.files import create_log_dir
from pueue.helper.socket import connect_client_socket, receive_data

from terminaltables import AsciiTable
from terminaltables.terminal_io import terminal_size


def execute_status(args):
    status = get_status()
    # First rows, showing daemon status
    if status['status'] == 'running':
        status['status'] = Color('{autogreen}' + '{}'.format(status['status']) + '{/autogreen}')
    elif status['status'] == 'paused':
        status['status'] = Color('{autoyellow}' + '{}'.format(status['status']) + '{/autoyellow}')

    if status['process'] == 'running' or status['process'] == 'paused':
        status['process'] = Color('{autogreen}' + '{}'.format(status['process']) + '{/autogreen}')

    print('Daemon: {}\nProcess status: {} \n'.format(status['status'], status['process']))

    # Handle queue data
    data = status['data']
    if isinstance(data, str):
        print(data)
    elif isinstance(data, dict):
        # Format incomming data to be compatible with Terminaltables
        formatted_data = []
        formatted_data.append(['Index', 'Status', 'Code',
                               'Command', 'Path', 'Start', 'End'])
        for key, entry in sorted(data.items(), key=operator.itemgetter(0)):
            formatted_data.append(
                [
                    '#{}'.format(key),
                    entry['status'],
                    '{}'.format(entry['returncode']),
                    entry['command'],
                    entry['path'],
                    entry['start'],
                    entry['end']
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
            # We have to subtract 14 because of table paddings
            left_space = math.floor((terminal_width[0] - customWidth[0] - customWidth[1] - customWidth[2] - customWidth[5] - customWidth[6] - 14)/2)

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
                    elif wrapped_string in ['errored', 'stopping', 'killing']:
                        wrapped_string = Color('{autored}' + '{}'.format(wrapped_string) + '{/autored}')
                elif j == 2:
                    if wrapped_string == '0' and wrapped_string != 'Code':
                        wrapped_string = Color('{autogreen}' + '{}'.format(wrapped_string) + '{/autogreen}')
                    elif wrapped_string != '0' and wrapped_string != 'Code':
                        wrapped_string = Color('{autored}' + '{}'.format(wrapped_string) + '{/autored}')

                table.table_data[i][j] = wrapped_string

        print(table.table)
    print('')


def execute_log(args):
    logPath = create_log_dir() + '/queue.log'
    logFile = open(logPath, 'r')
    print(logFile.read())


def execute_show(args):
    # Get current pueueSTDout file from tmp
    userName = getpass.getuser()
    stdoutFile = '/tmp/pueueStdout{}'.format(userName)
    stderrFile = '/tmp/pueueStderr{}'.format(userName)
    stdoutDescriptor = open(stdoutFile, 'r')
    stderrDescriptor = open(stderrFile, 'r')
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
            stdoutDescriptor.seek(0)
            message = stdoutDescriptor.read()
            stdscr.addstr(0, 0, message)
            stdscr.refresh()
            time.sleep(2)

        # Curses cleanup
        curses.nocbreak()
        stdscr.keypad(False)
        curses.echo()
        curses.endwin()
    else:
        print('Stdout output:\n')
        stdoutDescriptor.seek(0)
        print(stdoutDescriptor.read())
        print('\n\nStderr output:\n')
        stderrDescriptor.seek(0)
        print(stderrDescriptor.read())


def get_status():
    # Initialize socket, message and send it
    client = connect_client_socket()
    instruction = {'mode': 'status'}
    data_string = pickle.dumps(instruction, -1)
    client.send(data_string)

    response = receive_data(client)
    return response
