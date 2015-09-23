#!/usr/bin/env python3

import sys
import argparse

from daemon import *
from helper import removeSocket
from subcommands import *


def main():
    # Specifying commands
    parser = argparse.ArgumentParser(description='Pueue client/daemon')
    parser.add_argument('--daemon', action="store_true", help="Starts the pueue daemon")
    parser.add_argument('--stop', action="store_true", help="Stops the pueue daemon")
    subparsers = parser.add_subparsers(title='Subcommands', description='Various subcommands')

    # Add
    add_Subcommand = subparsers.add_parser('add', help='Adds a command to the queue')
    add_Subcommand.add_argument('command', type=str, help="The command to be added")
    add_Subcommand.set_defaults(func=executeAdd)

    # Remove
    remove_Subcommand = subparsers.add_parser('remove', help='Removes a spcific command from the queue')
    remove_Subcommand.add_argument('key', help="The index of the command to be deleted", type=int)
    remove_Subcommand.set_defaults(func=executeRemove)

    # Show
    show_Subcommand = subparsers.add_parser('show', help='Lists all commands in the queue')
    show_Subcommand.add_argument('--index', help='Shows the status of the command with the specified index, "Current" shows the current process')
    show_Subcommand.set_defaults(func=executeShow)

    # Pause
    pause_Subcommand = subparsers.add_parser('pause', help='Daemon will finishes the current command and pauses afterwards.')
    pause_Subcommand.set_defaults(func=executePause)

    # Stop
    stop_Subcommand = subparsers.add_parser('stop', help='Daemon will stop the current command and pauses afterwards.')
    stop_Subcommand.set_defaults(func=executeStop)

    # Start
    start_Subcommand = subparsers.add_parser('start', help='Daemon will stop the current command and pauses afterwards.')
    start_Subcommand.set_defaults(func=executeStart)

    # Exit
    exit_Subcommand = subparsers.add_parser('exit', help='Shuts the daemon down.')
    exit_Subcommand.set_defaults(func=executeExit)

    # Kills the current running process and starts the next
    kill_Subcommand = subparsers.add_parser('kill', help='Kills the current running process and starts the next one')
    kill_Subcommand.set_defaults(func=executeKill)

    args = parser.parse_args()

    if args.stop:
        executeStop(args)
    elif args.daemon:
        # daemon = Daemonize(app="pueue",pid='/tmp/pueue.pid', action=daemonMain)
        # daemon.start()
        try:
            daemonMain()
        except KeyboardInterrupt:
            print('Keyboard interrupt. Shutting down')
            removeSocket()
            sys.exit(0)
        except SystemExit:
            print('SystemExit. Shutting down')
            removeSocket()
            sys.exit(0)
    elif hasattr(args, 'func'):
        args.func(args)
    else:
        print('Invalid Command. Something strange happened, please check the help again')
