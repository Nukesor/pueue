import argparse

from pueue.client.factories import print_command_factory
from pueue.daemon.signals import signals

from pueue.client.displaying import (
    execute_status,
    execute_log,
    execute_show,
)

from pueue.client.manipulation import (
    execute_add,
)

# Specifying commands
parser = argparse.ArgumentParser(description='Pueue client/daemon')
parser.add_argument('--daemon', action='store_true', help='Starts the pueue daemon')
parser.add_argument(
    '--no-daemon', action='store_true',
    help='Starts the pueue daemon in the current terminal', dest='nodaemon'
)
parser.add_argument(
    '--stop-daemon', action='store_true',
    help='Daemon will shut down instantly. All running processes die', dest='stopdaemon')

parser.add_argument(
    '--root', type=str,
    help='The root directory for configs and logs. Defaults to home.')


# Initialize supbparser
subparsers = parser.add_subparsers(
    title='Subcommands', description='Various client')


# Status
status_subcommand = subparsers.add_parser(
    'status', help='List the daemon state and process queue.'
)
status_subcommand.set_defaults(func=execute_status)


# Configuration
config_parser = subparsers.add_parser(
    'config', help='Command for various configs.')

config_subparser = config_parser.add_subparsers(
    title='config subcommands', help='Subcommands to set various configs.')

# Configuration: Max process
max_processes_subcommand = config_subparser.add_parser(
    'maxProcesses', help='Set the amount of concurrent running processes.')
max_processes_subcommand.add_argument(
    'value', type=int,
    help="The amount of concurrent running processes."
)
max_processes_subcommand.set_defaults(
    func=print_command_factory('config'),
    option='maxProcesses',
)


# Show
show_subcommand = subparsers.add_parser('show', help='Shows the output of the currently running process')
show_subcommand.add_argument(
    '-w', '--watch', action='store_true',
    help='Get live output in a curses session. Like tail -f.'
)
show_subcommand.add_argument(
    '--key', '-k',  type=int,
    help='Show the output of a specific process.'
)

show_subcommand.set_defaults(func=execute_show)


# Logs
logs_subcommand = subparsers.add_parser(
    'log', help='Print the current log file to the command line.')
logs_subcommand.add_argument(
    'keys', type=int, nargs='*',
    help='Show the logs of the specified processes.'
)
logs_subcommand.set_defaults(func=execute_log)


# Add
add_subcommand = subparsers.add_parser(
    'add', help='Add a command to the queue.')
add_subcommand.add_argument(
    'command', type=str, nargs='+', help='The command to be added.')
add_subcommand.set_defaults(func=execute_add)

# Remove
remove_subcommand = subparsers.add_parser(
    'remove', help='Remove a specific command from the queue.')
remove_subcommand.add_argument(
    'keys', type=int, nargs='+',
    help='The index of the command to be deleted.')
remove_subcommand.set_defaults(func=print_command_factory('remove'))


# Switch
switch_subcommand = subparsers.add_parser(
    'switch', help='Switch two command in the queue.')
switch_subcommand.add_argument('first', help='The first command', type=int)
switch_subcommand.add_argument('second', help='The second command', type=int)
switch_subcommand.set_defaults(func=print_command_factory('switch'))


# Send
send_subcommand = subparsers.add_parser(
    'send', help='Send any input to the specified process.')
send_subcommand.add_argument('input', help='The input string', type=str)
send_subcommand.add_argument(
    'key', type=int,
    help='The index of the process, the message should be send to.'
)
send_subcommand.set_defaults(func=print_command_factory('send'))


# Reset
reset_subcommand = subparsers.add_parser(
    'reset', help='Kill the current command, reset queue and rotate logs.')
reset_subcommand.set_defaults(func=print_command_factory('reset'))


# Clear
clear_subcommand = subparsers.add_parser(
    'clear', help='Remove all `done` or `failed` commands from the queue. This will rotate logs as well.')
clear_subcommand.set_defaults(func=print_command_factory('clear'))


# Pause
pause_subcommand = subparsers.add_parser(
    'pause', help='Daemon will pause all running processes and stop to process the queue.')
pause_subcommand.add_argument(
    '-w', '--wait', action='store_true',
    help='Pause the daemon, but wait for current processes to finish.'
)
pause_subcommand.add_argument(
    'keys', type=int, nargs='*',
    help="The indices of the entries to be paused. The Daemon won't pause."
)
pause_subcommand.set_defaults(func=print_command_factory('pause'))


# Start
start_subcommand = subparsers.add_parser(
    'start', help='Daemon will start all paused processes and continue to process the queue.')
start_subcommand.add_argument(
    'keys', type=int, nargs='*',
    help="The indices of the entries to be started. The daemon will not start in case it's paused."
)
start_subcommand.set_defaults(func=print_command_factory('start'))


# Restart
restart_subcommand = subparsers.add_parser(
    'restart', help='Daemon will queue a finished process.')
restart_subcommand.add_argument(
    'keys', type=int, nargs='+',
    help='The indices of the entries to be restarted')
restart_subcommand.set_defaults(func=print_command_factory('restart'))


# Stash command
stash_subcommand = subparsers.add_parser(
    'stash', help="The specified command won't be processed by the daemon until it's enqueued.")
stash_subcommand.add_argument(
    'keys', type=int, nargs='+',
    help='The indices of the commands to be stashed.'
)
stash_subcommand.set_defaults(func=print_command_factory('stash'))


# Enqueue command
enqueue_subcommand = subparsers.add_parser(
    'enqueue', help="The specified command's status will be set to 'queued'.")
enqueue_subcommand.add_argument(
    'keys', type=int, nargs='+',
    help='The indices of the commands to be enqueued.'
)
enqueue_subcommand.set_defaults(func=print_command_factory('enqueue'))


def doesnt_represent_int(string):
    try:
        int(string)
        return False
    except:
        return True


case_sensitive_signals = list(signals.keys())
case_sensitive_signals += list(
    map(lambda x: x.upper(),
        filter(doesnt_represent_int, case_sensitive_signals))
)

# Kills the current running process
kill_subcommand = subparsers.add_parser(
    'kill', help='Kill all processes and pause the Daemon.')
kill_subcommand.add_argument(
    '-s', '--signal', choices=case_sensitive_signals,
    help='All running processes/the selected process will be removed from the queue.',
)
kill_subcommand.add_argument(
    'keys', type=int, nargs='*',
    help="The indices of the processes to be killed. The daemon won't pause."
)
kill_subcommand.set_defaults(
    func=print_command_factory('kill'),
    signal='sigint',
)
