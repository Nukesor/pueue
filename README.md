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
The queue will be continuously processed, even if you no longer have any active ssh sessions.

**Announcement:** v0.10.0 changed and broke a lot of things. Please check the release [Changelog](https://github.com/Nukesor/pueue/blob/master/CHANGELOG.md).

- [Features](https://github.com/Nukesor/pueue#features)
- [Why should I use it](https://github.com/Nukesor/pueue#why-should-i-use-it)
- [Installation](https://github.com/Nukesor/pueue#installation)
- [How to use it](https://github.com/Nukesor/pueue#how-to-use-it)
- [Advantages over Using a Terminal Multiplexer](https://github.com/Nukesor/pueue#advantages-over-using-a-terminal-multiplexer)

### Features

- Schedule commands that will be executed in their respective working directories.
- Easy output inspection.
- Interaction with running processes.
- Pause/resume tasks, when you need some processing power right NOW!
- Manipulation of the scheduled task order.
- Run multiple tasks at once. You can decide how many concurrent tasks you want to run.
- Group tasks. Each group acts as their own queue and can have several tasks running in parallel.
- A callback hook to, for instance, set up desktop notifications.
- A lot more. Check the -h options for each subcommand for detailed options.

Works on Linux and **partially** on MacOS and Windows. \
Check these issues to find out what's missing for MacOs (#115) and Windows (#114).

## Why should I use it

Consider this scenario: You have to unpack large amounts of data into various directories.
Usually something like this ends with 10+ open terminals/tmux sessions and an over-challenged hard drive.

Another scenario might be, that you want to re-encode 10 movies and each re-encode takes 10+ hours.
Creating a chained command with `&&`s isn't ergonomic at all and running that many re-encodes in parallel will break your CPU.

Pueue is specifically designed for these situations.\
You can schedule your task and continue on the same shell without waiting.
You can specify how many tasks should run in parallel and group tasks to maximize system resource utilization.\
Since everything is run by a daemon, you can simply log off your server and check on your tasks' progress whenever you want.\
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
If you feel like something is missing, please create an issue :).\
PRs are of course very welcome!

## Installation

There are four different ways to install Pueue.

#### Package Manager

<a href="https://repology.org/project/pueue/versions"><img align="right" src="https://repology.org/badge/vertical-allrepos/pueue.svg" alt="Packaging status"></a>

The preferred way to install Pueue is to use your system's package manager.  
This will usually deploy service files and completions automatically.  

Pueue has been packaged for a quite a few distributions, check the table on the right for more information.

#### Prebuild Binaries

Statically linked (if possible) binaries for Linux (incl. ARM), Mac OS and Windows are built on each release.

You can download the binaries for the client and the daemon (`pueue` and `pueued`) for each release on the [release page](https://github.com/Nukesor/pueue/releases).

Just download both binaries for your system, rename them to `pueue` and `pueued` and place them in your \$PATH/program folder.

#### Via Cargo

You'll need Rust version `>=1.48`

```bash
cargo install pueue
```

This will install pueue to `~/.cargo/bin/pueue`

#### From source

You'll need Rust version `>=1.48`

```bash
git clone git@github.com:Nukesor/pueue
cd pueue
cargo install --path .
```

This will install pueue to `~/.cargo/bin/pueue`

## How to use it

Check out the wiki to [get you started](https://github.com/Nukesor/pueue/wiki/Get-started) :).

There are also detailed sections for (hopefully) every important feature:

- [Configuration](https://github.com/Nukesor/pueue/wiki/Configuration)
- [Groups](https://github.com/Nukesor/pueue/wiki/Groups)
- [Miscellaneous](https://github.com/Nukesor/pueue/wiki/Miscellaneous)
- [Connect to remote](https://github.com/Nukesor/pueue/wiki/Connect-to-remote)

On top of that, there is a help option (-h) for all commands.

```text
Pueue client 0.10.0
Arne Beer <contact@arne.beer>
Interact with the Pueue daemon

USAGE:
    pueue [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -v, --verbose    Verbose mode (-v, -vv, -vvv)
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>    Path to a specific pueue config daemon, that should be used. This
                             ignores all other config files

SUBCOMMANDS:
    add            Enqueue a task for execution
    clean          Remove all finished tasks from the list (also clears logs)
    completions    Generates shell completion files. This can be ignored during normal
                   operations
    edit           Edit the command or path of a stashed or queued task.
                   This edits the command of the task by default.
    enqueue        Enqueue stashed tasks. They'll be handled normally afterwards
    follow         Follow the output of a currently running task. This command works like tail
                   -f
    group          Manage groups. By default, this will simply display all known groups
    help           Prints this message or the help of the given subcommand(s)
    kill           Kill specific running tasks or various groups of tasks
    log            Display the log output of finished tasks. Prints either all logs or only the
                   logs of specified tasks
    parallel       Set the amount of allowed parallel tasks
    pause          Pause either running tasks or specific groups of tasks.
                   By default, pauses the default queue and all its tasks.
                   A paused queue (group) won't start any new tasks.
    remove         Remove tasks from the list. Running or paused tasks need to be killed first
    reset          Kill all running tasks on user behalf, remove all tasks and reset max_task_id
    restart        Restart task(s). Identical tasks will be created and by default enqueued. By
                   default, a new task will be created
    send           Send something to a task. Useful for sending confirmations such as 'y\n'
    shutdown       Remotely shut down the daemon. Should only be used if the daemon isn't
                   started by a service manager
    start          Resume operation of specific tasks or groups of tasks.
                   By default, this resumes the default queue and all its tasks.
                   Can also be used force-start specific tasks.
    stash          Stashed tasks won't be automatically started. Either enqueue them, to be
                   normally handled or explicitly start them
    status         Display the current status of all tasks
    switch         Switches the queue position of two commands. Only works on queued and stashed
                   commands
    wait           Wait until tasks are finished. This can be quite useful for scripting. By
                   default, this will wait for all tasks in the default queue to finish. Note:
                   This will also wait for all tasks that aren't somehow 'Done'. Includes:
                   [Paused, Stashed, Locked, Queued, ...]
```

## Advantages over Using a Terminal Multiplexer

One of the most frequent questions is, why one should use Pueue, when there're terminal multiplexer such as Tmux or Screen.

My response is, that there're simply a lot of missing convenience features.\
Here are few examples of Pueue's basic functionality.

- The ability to queue commands and not start them all at once
- Specifying how many tasks should run in parallel
- Easy pausing/resuming of tasks
- Pretty and accessible task status overviews
- No need to attach to a multiple tmux sessions

There are a lot more built-in convenience features. You should read the [Wiki](https://github.com/Nukesor/pueue/wiki) for a detailed explanation.

Only using your shell's features is definitely possible!
However, in my opinion, having a tool that's specifically designed for managing tasks is just more efficient and fun.

One of my regular use cases is downloading lots of stuff. In this case I want:

- At most three parallel downloads, otherwise the other services on my server get starved.
- To see at first glance whether a download fails and easily edit and re-schedule it.
- An easy way to look at output.
- Everything to be in a uniform interface.
- It to look pretty and clear.
- To be able to pause/resume everything in case I need to some bandwidth right now.

I used tmux for this stuff all the time before writing Pueue.\
However, after using it for a really long time, it just kept feeling annoying and inconvenient.
Up to the point I could not bare it any longer and decided to write something that's better suited for such scenarios.

## Contributing

Feature requests and pull requests are very much appreciated and welcome!

Anyhow, please talk to me a bit about your ideas before you start hacking!
It's always nice to know what you're working on and I might have a few suggestions or tips :)

There's also the [Contribution Guide](https://github.com/Nukesor/pueue/blob/master/CONTRIBUTION_GUIDE.md), which is supposed to give you a brief overview and introduction into the project.

Copyright &copy; 2019 Arne Beer ([@Nukesor](https://github.com/Nukesor))

