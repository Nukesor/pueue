# Pueue

A queue for bash commands.

![Pueue](https://raw.githubusercontent.com/Nukesor/images/master/pueue.png)

## Why should I use it?

Everybody who lives on the commandline had this situation, when one needed to transfer huge amounts of data in different directories at the same time.

This normaly ends with about 10 open terminals/tmux sessions and an overchallenged hard drive.

But what if you could just queue those commands and they would be executed consecutively in their respective directory? Well that would be awesome!  
And this is just one possible application.  
Pueue is designed to execute long running tasks in the background, while not beeing bound to any terminal.  

If I got your attention, give it a try!  
If you think this is awesome, help me, join the development and create some PR's or suggest some improvements.

## Installation:

There are three different ways to install pueue.

1. Use an Arch Linux AUR package manager i.e Yaourt: `yaourt -S pueue-git` . This will deploy the service file automatically.
2. Install by using pip: `pip install pueue`.
3. Clone the repository and execute `python setup.py install`.

## How to use it:

There is a help option (-h) for all commands, but I'll list them here anyway.

`pueue --daemon` Starts the daemon. If the daemon finds a queue from a previous session it'll start in paused state!!  
`pueue --no-daemon` Starts the daemon in the current terminal.  
`pueue --stop-daemon` Daemon shuts down instantly. All running processes die.  

`pueue status` Shows the current queue, process and daemon state.  
If the queue is empty or the daemon is paused, the returcode of the last will be shown.  
`pueue show (--watch)` Shows the output of the currently running process.  
`pueue log` Prints the log of all executed commands.  

`pueue start` Daemon will start to process the queue.  
`pueue pause` Daemon will pause the current process and stops processing the queue.  
`pueue pause --wait` Waits for the current process to terminate on its own and pause the daemon afterwards.  
`pueue stop` Daemon will terminate the current process and pause.  
`pueue kill` KILLs the current process (kill -9) and pauses the daemon.  
`pueue reset` Removes all commands from the queue, kills the current process and resets the queue index to 0.  

`pueue add 'command'` Adds a command to the queue.  
`pueue remove index` Removes the command at #index.  
`pueue switch index1 index2` Switches the commands at #index1 and #index2.  

`pueue send 'input'` Sends a string to the subprocess's stdin. In case a process prompts for user input, you can use this to interact with the subprocess.
A `\n` may need to be attached to simulate an `Enter`. The stdin pipe is closed after every `send` command.

## Configs and Logs

The config file of pueue is located in `~/.config/pueue/pueue.ini`.

        [default]
        resumeafterstart = False
        stopaterror = True

        [log]
        logtime = 1209600

#### options

`stopAtError: True` Define if the demon should enter paused state, if a process in the queue fails.
`resumeAfterStart: False` If you want pueue to instantly resume a queue from the last session, set this value to `True`.

The logs of all your commands can be found in `~/.shared/pueue/*.log`. Old logs will be deleted after the time specified in your config.

## Utils

### Systemd
If you use systemd and don't install pueue with yaourt, place `pueue.service` in `/etc/systemd/user/`.  
Afterwards every user can start/enable it's own session with:  

        systemctl --user start pueue.service
        systemctl --user enable pueue.service

### ZSH Completion

If you use zsh, place `_pueue` in a folder, that is contained in your `FPATH` environment variable. This script will be propably added to zsh-users/zsh-completions, when it is finished.

## Libraries used

Regards to Robpol86 for providing the awesome `terminaltables` and `colorclass` libraries.
And thanks to thesharp for the extremly usefull `daemonize` library.

## Progress:
Pueue already works, but it needs debugging and propert testing. This is all in progress and will be done soon.

Copyright &copy; 2016 Arne Beer ([@Nukesor](https://github.com/Nukesor))
