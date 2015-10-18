# Pueue

A queue for bash commands.

![Pueue](https://raw.githubusercontent.com/Nukesor/images/master/pueue.png)

## Why should I use it?

Everybody who lives on the commandline had this situation, when he needed to transfer huge amounts of data in different directories at the same time.

This normaly ends with about 10 open terminals/tmux sessions and an overchallenged hard drive.

But what if you could just queue those commands and they would be executed consecutively in their respective directory? Well that would be awesome!  
And this is just one possible application.  
Pueue is designed to execute long running tasks in the background, while not beeing bound to any terminal.  

If I got your attention, give it a try!  
If you think this is awesome, help me, join the development and create some PR's or suggest some improvements.

## Installation:

Pueue can be installed by using `pip install pueue` or by cloning the repository and executing `python setup.py install`.

## How to use it:

There is a help option (-h) for all commands, but I'll list it here anyway.

`pueue --daemon` Starts the daemon. If the daemon finds a queue from a previous session it'll start in paused state!!  
`pueue --no-daemon` Starts the daemon in the current terminal.  
`pueue --stop-daemon` Daemon will shut down instantly. All running processes die.  

`pueue status` Shows the current queue, process and daemon state.  
If the queue is empty or the daemon is paused, the returcode of the last will be shown.
`pueue show (--watch)` Shows the output of the currently running process.  
`pueue log` Prints the log of all executed commands.  

`pueue start` Daemon will start to process the queue.  
`pueue pause` Daemon will pause after the current command terminated by it's own.  
`pueue stop` Daemon will terminate the current process and pause.  
`pueue kill` KILLs the current process (kill -9) and pauses the daemon.  
`pueue reset` Removes all commands from the queue, kills the current process and resets the queue index to 0.  

`pueue add 'command'` Adds a command to the queue.  
`pueue remove index` Removes the command at #index.  
`pueue switch index1 index2` Switches the commands at #index1 and #index2.  
