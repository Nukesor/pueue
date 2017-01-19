import argparse

from pueue.client.factories import print_command_factory

from pueue.client.displaying import (
    execute_status,
    execute_log,
    execute_show,
)

from pueue.client.manipulation import (
    execute_add,
    execute_remove,
    execute_restart,
    execute_pause,
    execute_stop,
    execute_kill,
    execute_switch,
    execute_send,
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
    help='The root directory for configs and logs. Used for testing')

# Initialze supbparser
subparsers = parser.add_subparsers(
    title='Subcommands', description='Various client')

# Status
status_Subcommand = subparsers.add_parser(
    'status', help='List the daemon state and process queue.'
)
status_Subcommand.set_defaults(func=execute_status)

# Show
show_Subcommand = subparsers.add_parser('show', help='Shows the output of the currently running process')
show_Subcommand.add_argument(
    '-w', '--watch', action='store_true',
    help='Get live output in a curses session. Like tail -f.'
)
show_Subcommand.set_defaults(func=execute_show)

# Logs
logs_Subcommand = subparsers.add_parser(
    'log', help='Print the current log file to the command line.')
logs_Subcommand.set_defaults(func=execute_log)

# Add
add_Subcommand = subparsers.add_parser(
    'add', help='Add a command to the queue.')
add_Subcommand.add_argument(
    'command', type=str, help='The command to be added.')
add_Subcommand.set_defaults(func=execute_add)

# Remove
remove_Subcommand = subparsers.add_parser(
    'remove', help='Remove a specific command from the queue.')
remove_Subcommand.add_argument(
    'key', help='The index of the command to be deleted.', type=int)
remove_Subcommand.set_defaults(func=execute_remove)

# Switch
switch_Subcommand = subparsers.add_parser(
    'switch', help='Switch two command in the queue.')
switch_Subcommand.add_argument('first', help='The first command', type=int)
switch_Subcommand.add_argument('second', help='The second command', type=int)
switch_Subcommand.set_defaults(func=execute_switch)

# Send
send_Subcommand = subparsers.add_parser(
    'send', help='Send any input to the specified process.')
send_Subcommand.add_argument('input', help='The input string', type=str)
send_Subcommand.add_argument(
    'key', type=int,
    help='The process this should be send to.'
)
send_Subcommand.set_defaults(func=execute_send)

# Reset
reset_Subcommand = subparsers.add_parser(
    'reset', help='Kill the current command, reset queue and rotate logs.')
reset_Subcommand.set_defaults(func=print_command_factory('reset'))

# Pause
pause_Subcommand = subparsers.add_parser(
    'pause', help='Daemon will pause all running processes and stop to process the queue.')
pause_Subcommand.add_argument(
    '-w', '--wait', action='store_true',
    help='Pause the daemon, but wait for current processes to finish.'
)
pause_Subcommand.add_argument(
    '-k', '--key', type=int,
    help='Pause a single process without pausing the Daemon.'
)
pause_Subcommand.set_defaults(func=execute_pause)

# Start
start_Subcommand = subparsers.add_parser(
    'start', help='Daemon will start all paused processes and continue to process the queue.')
start_Subcommand.add_argument(
    '-k', '--key', type=int,
    help="Start a single key. The daemon will not start in case it's paused."
)
start_Subcommand.set_defaults(func=print_command_factory('start'))

# Restart
restart_Subcommand = subparsers.add_parser(
    'restart', help='Daemon will queue a finished process.')
restart_Subcommand.add_argument(
    'key', help='The index of the entry to be restart', type=int)
restart_Subcommand.set_defaults(func=execute_restart)

# Kills the current running process
kill_Subcommand = subparsers.add_parser(
    'kill', help='Kill all processes and pause the Daemon.')
kill_Subcommand.add_argument(
    '-r', '--remove', action='store_true',
    help='All running processes/the selected process will be removed from the queue.'
)
kill_Subcommand.add_argument(
    '-k', '--key', type=int,
    help="Kills a single process. The daemon won't stop."
)
kill_Subcommand.set_defaults(func=execute_kill)

# Terminate the current running process and starts the next
stop_Subcommand = subparsers.add_parser(
    'stop', help='Stop all processes and pause the Daemon.')
stop_Subcommand.add_argument(
    '-r', '--remove', action='store_true',
    help='If this flag is set, the all running processes/the selected process will be removed from the queue.'
)
stop_Subcommand.add_argument(
    '-k', '--key', type=int,
    help="Stops a single process. The daemon won't stop."
)
stop_Subcommand.set_defaults(func=execute_stop)
