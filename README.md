# Pueue

![Pueue](https://raw.githubusercontent.com/Nukesor/images/master/pueue.png)

Pueue is a daemon designed for sequential and parallel execution of long running heavy load tasks. Not being bound to any terminal it is possible to check on your processes from every terminal or using the API. And the best part is that the queue will be processed by the daemon, even if you exit your ssh session.

It provides functionality for:
- Easy output inspection.
- Interaction with the running process
- Manipulation of the scheduled task order
- Running multiple tasks at once (You can decide how many concurrent tasks you want to run.)


## Why should I use it?

Pretty much everybody who lives on the command line had this situation, when one needed to unzip or transfer huge amounts of data in different directories.

This normally ends with about 10 open terminals/tmux sessions and an overchallenged hard drive.

Pueue is specifically designed for those situations. It executes long running tasks in their respective directories, without being bound to any terminal.  

Just a few possible applications:

- Long running compression tasks
- Movie encoding
- Copying stuff
- rsync tasks

If I got your attention, give it a try!  
If you think this is awesome, help me, join the development and create some PRs or suggest some improvements.

## Installation:

There are three different ways to install pueue.

1. Use an Arch Linux AUR package manager i.e Yaourt: `yaourt -S pueue-git` . This will deploy the service file automatically.
2. Install by using pip: `pip install pueue`.
3. Clone the repository and execute `python setup.py install`.

## How to use it:

There is a help option (-h) for all commands, but I'll list them here anyway.

`pueue --daemon` Starts the daemon. The daemon will try to load any queue from a previous session.  
`pueue --no-daemon` Start the daemon in the current terminal.  
`pueue --stop-daemon` Daemon will shut down after killing all processes.

`pueue status` Show the current state of processes and the daemon as well as the processing state of the queue.
`pueue show --watch --key $k` Show the output of `--key` or the oldest running process.  
    `show --watch` will continually show the stdout output of the subprocess in a `curses` session.  
    `show` without `--watch` will print the stderr, which can be useful if the subprocess prompts for user input (This is often piped to stderr).  

`pueue log $key` Print the output and status of all executed commands.  
`pueue start --key`This command has three different behaviours, depending on if and which a key is given:  
    1. If the key of a paused process is given, the process will be started (`SIGCONT`), this happens even if the daemon is paused.  
    1. If the key of a queued process is given, the process will be started, this happens even if the daemon is paused or the max amount of processes is exceeded.  
    3. If no key is given, the daemon will start to process the queue. This will start all paused processes (`SIGCONT`).  

`pueue pause --wait --key $k` This command has two different behaviours, depending on if a key is given:  
    1. If a key is given, pause the specified process by sending a `SIGSTOP`.  
    2. If no key is given, stop processing the queue and pause all running processes. If the `--wait` flag is set, the daemon will pause, but all running processes will finish on their own.  

`pueue restart` Enqueue a finished process.  
`pueue stop -r --key` This command has two different behaviours, depending on if a key is given:  
    1. If a key is given, terminate the specified process. If `-r` is provided this process will be removed from the queue.  
    2. If no key is given, terminate all running processes (`kill`) and pause the daemon.  

`pueue kill -r --key` This command has two different behaviours, depending on if a key is given:  
    1. If a key is given, KILL the specified process (`kill -9`). If `-r` is provided the current running process will be removed from the queue.  
    2. If no key is given, KILL all running processes (`kill -9`) and pause the daemon. If `-r` is provided this process will be removed from the queue.  

`pueue reset` Remove all commands from the queue, kill the current process and reset the queue index to 0.  
`pueue add 'command'` Add a command to the queue.  
`pueue remove index` Remove the command at #index.  
`pueue switch index1 index2` Switch the commands at position #index1 and #index2.  
`pueue send 'input'` Send a string to the subprocess's stdin. In case a process prompts for user input, you can use this to interact with the subprocess.
The stdin pipe is flushed after every `send` command. To simulate a `\n` you need to add a newline in your string:

        pueue send 'y
        '

`pueue config` This command allows to set different config values without editing the config file and restarting the daemon. Look at `pueue config -h` for more information.


## Configs

The configuration file of pueue is located in `~/.config/pueue/pueue.ini`.

        [default]
        stopAtError = True
        resumeAfterStart = False
        maxProcesses = 1

        [log]
        logTime = 1209600

`stopAtError = True` Determines if the demon should enter paused state, if a process in the queue fails.
`resumeAfterStart = False` If you want pueue to instantly resume a queue from the last session, set this value to `True`.
`maxProcesses = 1` Determines how many tasks should be processed concurrently.

`logTime = 1209600`  Old logs will be deleted after the time specified in your config.

## Logs 

All logs can be found in `~/.shared/pueue/*.log`. Logs of previous pueue session will be rotated and contain a timestamp in the name.  
In case the daemon fails or something goes wrong, there is a separate log for the daemon at `~/.shared/pueue/daemon.log`.
If the daemon crashes, please send the stack trace from this log!


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
