# Pueue

[![GitHub Actions Workflow](https://github.com/nukesor/pueue/workflows/Test%20build/badge.svg)](https://github.com/Nukesor/pueue/actions)
[![Crates.io](https://img.shields.io/crates/v/pueue)](https://crates.io/crates/pueue)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![AUR](https://img.shields.io/aur/version/pueue.svg)](https://aur.archlinux.org/packages/pueue/)
[![Downloads](https://img.shields.io/github/downloads/nukesor/pueue/total.svg)](https://github.com/nukesor/pueue/releases)
[![Patreon](https://github.com/Nukesor/images/blob/master/patreon-donate-blue.svg)](https://www.patreon.com/nukesor)
[![Paypal](https://github.com/Nukesor/images/blob/master/paypal-donate-blue.svg)](https://www.paypal.me/arnebeer/)
<!---[![dependency status](https://deps.rs/repo/github/nukesor/pueue/status.svg)](https://deps.rs/repo/github/nukesor/pueue) --->

![Pueue](https://raw.githubusercontent.com/Nukesor/images/master/pueue.gif)

Pueue is a command-line task management tool for sequential and parallel execution of long-running tasks.  

Simply put, it's a tool that processes a queue of shell commands.
On top of that, there are a lot of convenience features and abstractions.

Since Pueue is not bound to any terminal, you can control your tasks from any terminal on the same machine.
The queue will be continuously processed, even if you no longer have any active ssh session.

### Features:

- Schedule commands that will be executed in their respective working directories.
- Easy output inspection.
- Interaction with running processes.
- Pause/resume tasks, when you need some processing power right NOW!
- Manipulation of the scheduled task order.
- Running multiple tasks at once (You can decide how many concurrent tasks you want to run).
- Grouping tasks. Each group acts as their own queue and can have several tasks running in parallel.
- A callback hook to, for instance, set up desktop notifications.
- A lot more. Check the -h options for each subcommand for detailed options.

Works on Linux, MacOS and partially on Windows.

**Disclaimer:** Windows and MacOS don't yet support the full feature set:

#### Windows

The Windows version needs someone to implement process handling!
Right now there's pretty much no Windows specific code which means:

- Pausing/resuming commands doesn't work for now.
- Sending signals to processes' children doesn't work.
- We cannot ensure that there aren't dangling processes on `kill`.
- Pueue only supports `powershell` for executing commands, keep this in mind when writing commands.

#### MacOs

The `psutil` library, which is used for MacOS is a horrible mess.
It's missing a lot of missing features and has an extremely bad code style.

Someone has to add a proper replacement and re-implement the MacOs specific process handling code.
Until then:

- Sending signals to processes' children doesn't work.
- We cannot ensure that there aren't dangling processes on `kill`.

## Why should I use it

Consider this scenario: You have to unpack large amounts of data into various directories.
Usually something like this ends with 10+ open terminals/tmux sessions and an over-challenged hard drive.

Another scenario might be, that you want to re-encode 10 movies and each re-encode takes 10+ hours.
Creating a chained command with `&&`s isn't ergonomic at all and running that many re-encodes in parallel will break your CPU.  

Pueue is specifically designed for these situations.

You can schedule your task and continue on the same shell without waiting.
You can specify how many tasks should run in parallel and group tasks to maximize system resource utilization.

Since everything is run by a daemon, you can simply log off your server and check on your tasks' progress whenever you want.

Heck, you can even set up desktop notifications to get notified or execute parameterized commands every time a task finishes.

**A few possible applications:**

- Copying large amounts of data
- Machine learning
- Compression tasks
- Movie encoding
- `rsync` tasks
- Anything that takes longer than 5 minutes

Pueue made at least my life a lot easier on many occasions.

If you like the project, feel free to give it at try!  
If you feel like something is missing, please create an issue :).

PRs are of course very welcome!

## Installation

There are four different ways to install Pueue.

#### Package Manager

Use your system's package manager.  
This will usually deploy service files and completions automatically.  
Pueue has been packaged for:

- Arch Linux's AUR: e.g. `yay -S pueue`.  
- NixOS
- Homebrew

#### Prebuild Binaries

Statically linked (if possible) binaries for Linux (incl. ARM), Mac OS and Windows are built on each release.

You can download the binaries for the client and the daemon (`pueue` and `pueued`) for each release on the [release page](https://github.com/Nukesor/pueue/releases).

Just download both binaries for your system, rename them to `pueue` and `pueued` and place them in your $PATH/program folder.

#### Via Cargo

You'll need Rust version `>=1.39`

```bash
cargo install pueue
```

This will install pueue to `~/.cargo/bin/pueue`

#### From Source

You'll need Rust version `>=1.39`

```bash
git clone git@github.com:Nukesor/pueue
cd pueue
cargo install --path .
```

This will install pueue to `~/.cargo/bin/pueue`

## Starting the Daemon

### Local

Just run `pueued` anywhere on your commandline. It'll exit if you close the terminal, though.

### Background

To fork `pueued` into the background, add the `-d` or `--daemonize` flag. E.g. `pueued -d`. \
The daemon can be then shut down using the client: `pueue shutdown`

### Systemd

If you use Systemd and don't install Pueue with a package manager, place `pueued.service` in `/etc/systemd/user/`.  
Afterward, every user can start/enable their own session with:  

```bash
systemctl --user start pueued.service
systemctl --user enable pueued.service
```


## How to use it

**Adding Commands:**

To add a command just write: `pueue add sleep 60`\
If you want to add flags to the command, you can either:

- add `--` => `pueue add -- ls -al`
- surround the command with a string `pueue add 'ls -al'`

The command will then be added and scheduled for execution, as if you executed it right now and then.

For normal operation it's recommended to add an alias to your shell's rc.\
E.g.: `alias pad='pueue add --'`

Surrounding a command with quotes is also required, if your command contains escaped characters.\
For instance `pueue add ls /tmp/long\ path` will result in the execution of `sh -c ls /tmp/long path`, which will then break, as the escaped space is not passed to Pueue.

**See what's going on:**

To get the status of currently running commands, just type `pueue status`.

To look at the current output of a command use `pueue log` or `pueue log $task_id`.

If you want to follow the output of a running command use `git follow $task_id`.
To follow stderr, use the `-e` flag.

**Manipulate multiple tasks at once:**

Most commands can be executed on multiple tasks.
For instance, you can look at specific logs like this `pueue log 0 1 2 3 15 19`.

This also works with your shell's range parameter, e.g. `pueue log {0..3} 15 19`.


**Pitfalls:**

To avoid common pitfalls, please read the [FAQ Section](https://github.com/Nukesor/pueue/blob/master/FAQ.md).

There is a help option (-h) for all commands.

```text
Pueue client 0.5.0
Arne Beer <contact@arne.beer>
Interact with the Pueue daemon

USAGE:
    pueue [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose mode (-v, -vv, -vvv)

OPTIONS:
    -p, --port <port>    The port for the daemon. Overwrites the port in the config file

SUBCOMMANDS:
    add            Enqueue a task for execution
    clean          Remove all finished tasks from the list (also clears logs)
    completions    Generates shell completion files. This can be ignored during normal operations
    edit           Edit the command or path of a stashed or queued task.
                   This edits the command of the task by default.
    enqueue        Enqueue stashed tasks. They'll be handled normally afterwards
    follow         Follow the output of a currently running task. This command works like `tail -f`
    group          Manage groups. Without any flags, this will simply display all known groups
    help           Prints this message or the help of the given subcommand(s)
    kill           Kill specific running tasks or various groups of tasks
    log            Display the log output of finished tasks
    parallel       Set the amount of allowed parallel tasks
    pause          Pause either running tasks or specific groups of tasks.
                   Without any parameters, pauses the default queue and all it's tasks.
                   A paused queue (group) won't start any new tasks.
                   Everything can be resumed with `start`.
    remove         Remove tasks from the list. Running or paused tasks need to be killed first
    reset          Kill all running tasks, remove all tasks and reset max_task_id
    restart        Restart task(s). Identical tasks will be created and instantly queued (unless specified
                   otherwise)
    send           Send something to a task. Useful for sending confirmations ('y\n')
    shutdown       Remotely shut down the daemon. Should only be used if the daemon isn't started by a service
                   manager
    start          Resume operation of specific tasks or groups of tasks.
                   Without any parameters, resumes the default queue and all it's tasks.
                   Can also be used force specific tasks to start.
    stash          Stashed tasks won't be automatically started. Either `enqueue` them, to be normally handled or
                   explicitly `start` them
    status         Display the current status of all tasks
    switch         Switches the queue position of two commands. Only works on queued and stashed commands
```

## Configs

The configuration file of Pueue is located in `$CARGO_HOME/pueue.yml`.  
The default will be generated after starting pueue once.

```yaml
---
client:
  daemon_port: "6924"
  secret: "your_secret"
  read_local_logs: true

daemon:
  pueue_directory: /home/$USER/.local/share/pueue
  default_parallel_tasks: 1
  pause_on_failure: false
  port: "6924"
  secret: "your_secret"
  callback: ""Task {{ id }}\nCommand: {{ command }}\nPath: {{ path }}\nFinished with status '{{ result }}'\""
  groups:
    cpu: 1
```

**Client**:

- `daemon_port` The port the client tries to connect to.  
- `secret` The secret, that's used for authentication
- `read_local_logs` If the client runs as the same user (and on the same machine) as the daemon, logs don't have to be sent via the socket but rather read directly.

**Daemon**:

- `pueue_directory` The location Pueue uses for its intermediate files and logs.
- `default_parallel_tasks` Determines how many tasks should be processed concurrently.  
- `pause_on_failure` If set to `true`, the daemon stops starting new task as soon as a single task fails. Already running tasks will continue.
- `port` The port the daemon should listen to.  
- `secret` The secret, that's used for authentication
- `callback` The command that will be called after a task finishes. Can be parameterized
- `groups` This is a list of the groups with their amount of allowed parallel tasks. It's advised to not manipulate this manually, but rather use the `group` subcommand to create and remove groups.

## Logs

All logs can be found in `${pueue_directory}/logs`.
Logs of previous Pueue sessions will be created whenever you issue a `reset` or `clean`.  
In case the daemon fails or something goes wrong, the daemon will print to `stdout`/`stderr`.
If the daemon crashes or something goes wrong, please set the debug level to `-vvvv` and create an issue with the log!

If you want to dig right into it, you can compile and run it yourself with a debug build.
This would help me a lot!

## Utilities

### Groups

Grouping tasks can be useful, whenever your tasks utilize different system resources.  
A possible scenario would be to have an `io` group for tasks that copy large files, while your cpu-heavy (e.g. reencoding) tasks are in a `cpu` group.
The parallelism setting of `io` could then be set to `1` and `cpu` be set to `2`.  

As a result, there'll always be a single task that copies stuff, while two tasks try to utilize your cpu as good as possible.

This removes the problem of scheduling tasks in a way that the system might get slow.
At the same time, you're able to maximize resource utilization.

### Aliases

To get basic aliasing, simply put a `pueue_aliases.yml` besides your `pueue.yml`.
Its contents should look something like this:

```yaml
ls: "ls -ahl"
rsync: "rsync --recursive --partial --perms --progress"
```

When adding a command to pueue, the **first** word will then be checked for the alias.
This means, that for instance `ls ~/ && ls /` will result in `ls -ahl ~/ && ls /`.

If you want multiple aliases in a single task, it's probably best to either create a task for each command or to write a custom script.

### Callbacks

You can specify a callback that will be called every time a task finishes.
The callback can be parameterized with some variables.  

These are the available variables that can be used to create a command:

- `{{ id }}`
- `{{ command }}`
- `{{ path }}`
- `{{ result }}` (Success, Killed, etc.)
- `{{ group }}`

Example callback:

```yaml
    callback: "notify-send \"Task {{ id }}\nCommand: {{ command }}\nPath: {{ path }}\nFinished with status '{{ result }}'\""
```

### Shell completion files

Shell completion files can be created on the fly with `pueue completions $shell $directory`.
There's also a `build_completions.sh` script, which creates all completion files in the `utils/completions` directory.

### JSON Support

The Pueue client `status` and `log` commands support JSON output with the `-j` flag.
This can be used to easily incorporate it into window manager bars, such as i3bar.

## Scripting

When calling pueue commands in a script, you might need to sleep for a short amount of time for now.
The pueue server processes requests asynchronously, whilst the TaskManager runs it's own update loop with a small sleep.
(The TaskManager handles everything related to starting, stopping and communicating with processes.)

A sleep in scripts will probably become irrelevant, as soon as this bug in rust-lang is fixed: [issue](https://github.com/rust-lang/rust/issues/39364)

# Contributing

Feature requests and pull requests are very much appreciated and welcome!

Anyhow, please talk to me a bit about your ideas before you start hacking!
It's always nice to know what you're working on and I might have a few suggestions or tips :)

There's also the [Contribution Guide](https://github.com/Nukesor/pueue/blob/master/CHANGELOG.md), which is supposed to give you a brief overview and introduction into the project.

Copyright &copy; 2019 Arne Beer ([@Nukesor](https://github.com/Nukesor))
