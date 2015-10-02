# Pueue

Pueue is a queue for bash commands.

![Pueue](https://raw.githubusercontent.com/Nukesor/images/master/pueue.png)

## Why should i use it?

Everybody who lives on the commandline had this situation, when he needed to transfer huge amounts of data in different directories at the same time.

This normaly ends with about 10 open terminals/tmux sessions and an overchallenged hard drive.

But what if you could just queue those commands and they would be executed consecutively in their respective directory? Well that would be awesome!
Pueue is supposed to do exactly that. And this is just one possible application.

Pueue is still under heavy development, but it already supports all basic functions as well as logging of all executed commands.
And there will be even more stuff, like multi user usage or fancy real time stdout displaying of the current process.

If i got your attention, clone it and give it a try!
If you think this is awesome, help me, join the development and create some PR's.

## Big TODO's:

- Proper Daemonization
- Testing!!! Includes refactoring for easier unit testing
- Realtime stdout/stderr watching
- Multi user usage of a single daemon?


## How to use it:

There is a help option (-h) for all commands, but I'll list it here anyway.

`pueue --daemon` Starts the daemon. The daemonization doesn't work on it's own yet, but there are alternatives e.g. systemd.
If the daemon finds a queue from the previous session it'll start in paused state!!
`pueue exit` Shuts the daemon down.
`pueue start` Daemon will start to process the queue.
`pueue pause` Daemon will pause after the current command terminated by it's own.
`pueue stop` Daemon will terminate the current process and pause.
`pueue reset` Removes all commands from the queue, kills the current process and resets the queue index to 0.

`pueue add 'command'` Adds an command to the queue.
`pueue remove index` Removes the command at #index.
`pueue kill` KILLs the current command (kill -9) and pauses the daemon.

`pueue show` Shows the current queue, process and the daemon state.
`pueue log` Prints the log of all executed commands.

