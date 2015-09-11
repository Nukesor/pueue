#!/bin/python3
import argparse
from daemonize import Daemonize

from daemon import *
from subcommands import *

# Specifying commands
parser = argparse.ArgumentParser(description='Pueue client/daemon')
parser.add_argument('--daemon', action="store_true", help="Starts the actual pueue daemon")
subparsers = parser.add_subparsers(title='Subcommands',description='Various subcommands')

# Add
add_Subcommand = subparsers.add_parser('add', help='Adds a command to the queue')
add_Subcommand.add_argument('command', type=str, help="The command to be added")
add_Subcommand.set_defaults(func=executeAdd)

# Remove
remove_Subcommand = subparsers.add_parser('remove', help='Removes a spcific command from the queue')
remove_Subcommand.add_argument('removeIndex', help="The index of the command to be deleted", type=int)

# Show
show_Subcommand = subparsers.add_parser('show', help='Lists all commands in the queue')
show_Subcommand.add_argument('--index', help='Shows the status of the command with the specified index')



args = parser.parse_args()

if args.daemon:
    #daemon = Daemonize(app="pueue",pid='/tmp/pueue.pid', action=daemonMain)
    #daemon.start()
    daemonMain()
elif args.func:
    args.func(args)

