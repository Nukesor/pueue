- [Start the daemon](#start-the-daemon)
    * [Systemd](#systemd)
- [How to use the client](#how-to-use-the-client)
    * [Adding commands](#adding-commands)
    * [See what's going on](#see-whats-going-on)
    * [Parallel tasks](#parallel-tasks)
    * [Pause, resume and start tasks](#pause-resume-and-start-tasks)
    * [Manipulate multiple tasks at once](#manipulate-multiple-tasks-at-once)
    * [Dependencies, delays, immediate](#dependencies-delays-immediate)
- [Common pitfalls](#common-pitfalls)
    * [A command does not behave like expected](#a-command-does-not-behave-like-expected)
    * [The command formatting seems to be broken](#the-command-formatting-seems-to-be-broken)
    * [A process waits for input](#a-process-waits-for-input)
    * [My shell aliases do not work](#my-shell-aliases-do-not-work)
    * [Display not found](#display-not-found)


## Start the Daemon

<a name="headers"/>

Before you can use the `pueue` client, you have to start the daemon.

**Local:**
The daemon can be be run in the current shell.
Just run `pueued` anywhere on your commandline. It'll exit if you close the terminal, though.

**Background:**
To fork and run `pueued` into the background, add the `-d` or `--daemonize` flag. E.g. `pueued -d`. \
The daemon can always be shut down using the client command `pueue shutdown`.

### Systemd

If you use Systemd and don't install Pueue with a package manager, place `pueued.service` in `/etc/systemd/user/`.  
Afterward, every user can start/enable their own session with:

```bash
systemctl --user start pueued.service
systemctl --user enable pueued.service
```

## How to use the client

### Adding commands

To add a command just write: `pueue add sleep 60`\
If you want to add flags to the command, you can either:

- add `--` => `pueue add -- ls -al`
- surround the command with a string `pueue add 'ls -al'`

The command will then be added and scheduled for execution, as if you executed it right now and then.
For normal operation it's recommended to add an alias to your shell's rc.\
E.g.: `alias pad='pueue add --'`

Surrounding a command with quotes is also required, if your command contains escaped characters.\
For instance `pueue add ls /tmp/long\ path` will result in the execution of `sh -c ls /tmp/long path`, which will then break, as the escaped space is not passed to Pueue.

### See what's going on

To get the status of currently running commands, just type `pueue status`.\
To look at the current output of a command use `pueue log` or `pueue log $task_id`.\
If you want to follow the output of a running command use `git follow $task_id`.
To follow stderr, use the `-e` flag.

### Parallel tasks

By default pueue only executes a single task at a time.
This can be changed in the configuration file, but also on-demand during runtime.
Just use the `parallel` subcommand, e.g. `pueue parallel 3`.
Now there'll always be up to three tasks running in parallel.

### Pause, resume and start tasks

Without any parameters, the `pause` subcommand pauses all running tasks and the daemon itself.
A pause daemon won't start any new tasks, until it's started again.

To resume normal operation, just write `pueue start`.
This will continue all paused tasks and the daemon will continue starting tasks.

However, you can also pause specific tasks, without affecting the daemon or any other tasks.
Just add the id of the this task as a parameter, e.g. `pueue pause 1`.
It can be resumed the same way with the `start` command.

`start` can also force tasks to be started.
This will ignore any limitations on parallel tasks and just spawn the process.


### Manipulate multiple tasks at once

Most commands can be executed on multiple tasks at once.
For instance, you can look at specific logs like this:\
`pueue log 0 1 2 3 15 19`.

This also works with your shell's range parameter, e.g. `pueue log {0..3} 15 19`.

### Dependencies, delays, immediate

There are more ways to specify when a command should be executed.
Check the help text of the `add` subcommand to see all options.

As an example, you can

- Specify dependencies. The task will only be executed if all dependencies were successful.
- Set a delay. The task will be scheduled after e.g. 5 hours.
- force a start. The task will be started immediately.


## Common pitfalls

## A command does not behave like expected

**First thing to do:** Try to run the command without adding it to Pueue.
If this fails, it's not a problem with Pueue.

**Second thing** to do when debugging any problems with running/failing processes, is to look at the process output:

This can be done via `pueue log $task_id`.
You can also get a live view of the output with `pueue follow $task_id`.
Add the `-e` flag, if you want to see the error output.

### The command formatting seems to be broken

Pueue takes your input and uses it exactly as is to create a new `bash -c $command` in the background.  
If your command contains spaces or characters that need escaping, you might need to encapsulate it into a string:

```bash
pueue add -- ls -al "/tmp/this\ is\ a\ test\ directory"
```

Without quotes, the character escaping won't be transferred to the `bash -c $command`, as it's already removed by calling it from the current shell.


### A process waits for input

Sometimes some process waits for input. For instance, a package manager may wait for confirmation (`y/n`).

In this case you can send the desired input to the process via:

```bash
pueue send "y
"
```

This can be also be avoided by issuing the command with something like a `-y` flag (if the program allows something like this).

### My shell aliases don't work

Pueue doesn't support aliases in shell's `.*rc` files, since that's pretty tricky.
That's why Pueue brings it's own aliasing.
Check the Readme on how to use it.

### Display not found

All programs that require some kind of display/window manager won't work, as the tasks are executed in the background.

Don't use Pueue for commands that won't work in a non-visual environment.

### My shell aliases do not work

Pueue doesn't support aliases in shell's `.*rc` files, since that's pretty tricky.
That's why Pueue brings it's own aliasing.
Check the Readme on how to use it.

