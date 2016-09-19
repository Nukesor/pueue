# Pueue

A queue for bash commands.

![Pueue](https://raw.githubusercontent.com/Nukesor/images/master/pueue.png)

## Why should I use it?

Everybody who lives on the command line had this situation, when one needed to unzip or transfer huge amounts of data in different directories.

This normally ends with about 10 open terminals/tmux sessions and an overchallenged hard drive.

But what if you could just queue those commands and they would be executed consecutively in their respective directory? Well that would be awesome!  
And this is just one possible application.  
Pueue is designed to execute long running tasks in the background, while not being bound to any terminal.  

If I got your attention, give it a try!  
If you think this is awesome, help me, join the development and create some PRs or suggest some improvements.

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

`pueue status` Shows the current state of the process and daemon as well as the processing state of the queue.
`pueue show (--watch)` Shows the output of the currently running process. `show --watch` will only show the stdout output of the subprocess.
`show` on it's own will also print the stderr, which can be useful, if the subprocess prompts for user input (This is often piped to stderr).  
`pueue log` Prints the output and statuses of all executed commands.  

`pueue start` Daemon will start to process the queue. This starts any paused processes as well (`SIGCONT`).  
`pueue pause ` Stop processing the queue and pause the underlying process by sending a `SIGSTOP`.  
`pueue restart` Enqueue a finished process.  
`pueue stop (-r)` Terminate the current process (`kill`) and pause the daemon afterwards. If `-r` is provided the current running process will be removed from the queue.  
`pueue kill (-r)` KILL the current process (`kill -9`) and pause the daemon afterwards. If `-r` is provided the current running process will be removed from the queue.  
`pueue reset` Remove all commands from the queue, kill the current process and reset the queue index to 0.  

`pueue add 'command'` Add a command to the queue.  
`pueue remove index` Remove the command at #index.  
`pueue switch index1 index2` Switch the commands at position #index1 and #index2.  

`pueue send 'input'` Send a string to the subprocess's stdin. In case a process prompts for user input, you can use this to interact with the subprocess.
The stdin pipe is flushed after every `send` command. To simulate a `\n` you need to add a newline in your string:

        pueue send 'y
        '

## Configs and Logs

The configuration file of pueue is located in `~/.config/pueue/pueue.ini`.

        [default]
        resumeafterstart = False
        stopaterror = True

        [log]
        logtime = 1209600

#### options

`stopAtError = True` Define if the demon should enter paused state, if a process in the queue fails.
`resumeAfterStart = False` If you want pueue to instantly resume a queue from the last session, set this value to `True`.

`logtime = 1209600` The logs of all your commands can be found in `~/.shared/pueue/*.log`. Old logs will be deleted after the time specified in your config.

## Utils

### Systemd
If you use systemd and don't install pueue with yaourt, place `pueue.service` in `/etc/systemd/user/`.  
Afterwards every user can start/enable it's own session with:  

        systemctl --user start pueue.service
        systemctl --user enable pueue.service

### ZSH Completion

If you use zsh, place `_pueue` in a folder, that is contained in your `FPATH` environment variable. This script will be probably added to zsh-users/zsh-completions, when it is finished.

## Libraries used

Regards to Robpol86 for providing the awesome `terminaltables` and `colorclass` libraries.
And thanks to thesharp for the extremely useful `daemonize` library.

## Progress:
Pueue already works and is frequently used. There might be some small bugs, but I didn't encounter something serious in quite a while.

Copyright &copy; 2016 Arne Beer ([@Nukesor](https://github.com/Nukesor))
