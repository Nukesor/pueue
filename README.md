# Pueue

[![GitHub Actions Workflow](https://github.com/nukesor/pueue/workflows/Test%20build/badge.svg)](https://github.com/Nukesor/pueue/actions)
[![Crates.io](https://img.shields.io/crates/v/pueue)](https://crates.io/crates/pueue)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/github/downloads/nukesor/pueue/total.svg)](https://github.com/nukesor/pueue/releases)
[![codecov](https://codecov.io/gh/nukesor/pueue/branch/main/graph/badge.svg)](https://codecov.io/gh/nukesor/pueue)

![Pueue](https://raw.githubusercontent.com/Nukesor/images/main/pueue-v2.0.0.gif)

Pueue is a command-line task management tool for sequential and parallel execution of long-running tasks.

Simply put, it's a tool that **p**rocesses a q**ueue** of shell commands.
On top of that, there are a lot of convenient features and abstractions.

Since Pueue is not bound to any terminal, you can control your tasks from any terminal on the same machine.
The queue will be continuously processed, even if you no longer have any active ssh sessions.

**Pueue is considered feature-complete :tada:.**
All features that were planned have been added and only minor improvements, bug-fixes and regular maintenance work will get merged.

- [Features](https://github.com/Nukesor/pueue#features)
- [Installation](https://github.com/Nukesor/pueue#installation)
- [How to use it](https://github.com/Nukesor/pueue#how-to-use-it)
- [Similar Projects](https://github.com/Nukesor/pueue#similar-projects)
- [Design Goals](https://github.com/Nukesor/pueue#design-goals)
- [Contributing](https://github.com/Nukesor/pueue#contributing)

## Features

- Scheduling
    * Add tasks as you go.
    * Run multiple tasks at once. You decide how many tasks should run concurrently.
    * Change the order of the scheduled tasks.
    * Specify dependencies between tasks.
    * Schedule tasks to run at a specific time.
- Process interaction
    * Easy output inspection.
    * Send input to running processes.
    * Pause/resume tasks, when you need some processing power right NOW!
- Task groups (multiple queues)
    * Each group can have several tasks running in parallel.
    * Pause/start tasks by a group.
- Background process execution
    * The `pueued` daemon runs in the background. No need to be logged in.
    * Commands are executed in their respective working directories.
    * The current environment variables are copied when adding a task.
    * Commands are run in a shell which allows the full feature set of shell coding.
- Consistency
    * The queue is always saved to disk and restored on kill/system crash.
    * Logs are persisted onto the disk and survive a crash.
- Miscellaneous
    * A callback hook to, for instance, set up desktop notifications.
    * JSON output for `log` and `status` if you want to display info about tasks in another program.
    * A `wait` subcommand to wait for specific tasks, a group (or everything) to finish.
- A lot more. Check the -h options for each subcommand for detailed options.
- Cross Platform
    * Linux is fully supported and battle-tested.
    * MacOS is fully supported and on par with Linux.
    * Windows is fully supported and working fine for quite a while.
- [Why should I use it](https://github.com/Nukesor/pueue/wiki/FAQ#why-should-i-use-it)
- [Advantages over Using a Terminal Multiplexer](https://github.com/Nukesor/pueue/wiki/FAQ#advantages-over-using-a-terminal-multiplexer)


## What Pueue is **not**

Pueue is **not** designed to be a programmable (scriptable) task scheduler/executor.

The focus of `pueue` lies on human interaction, i.e. it's supposed to be used by a real person on some kind of OS.
See [the Design Goals section](#design-goals)

Due to this, the feature set of `pueue` and `pueued` as well as their implementation and architecture have been kept simple by design!
Even though it can be scripted to some degree, it hasn't been built for this and there's no official support!

There's definitely the need for a complex task scheduler/executor with advanced API access and scheduling options, but this is the job for another project, as this is not what pueue has been built for.

## Installation

There are a few different ways to install Pueue.

#### Package Manager

<a href="https://repology.org/project/pueue/versions"><img align="right" src="https://repology.org/badge/vertical-allrepos/pueue.svg" alt="Packaging status"></a>

The preferred way to install Pueue is to use your system's package manager.
This will usually deploy service files and completions automatically.

Pueue has been packaged for quite a few distributions, check the table on the right for more information.

#### Prebuild Binaries

Statically linked (if possible) binaries for Linux (incl. ARM), Mac OS and Windows are built on each release. \
You can download the binaries for the client and the daemon (`pueue` and `pueued`) for each release on the [release page](https://github.com/Nukesor/pueue/releases). \
Just download both binaries for your system, rename them to `pueue` and `pueued` and place them in your \$PATH/program folder.

#### Via Cargo

Pueue is built for the current `stable` Rust version.
It might compile on older versions, but this isn't tested or officially supported.

```bash
cargo install --locked pueue
```

This will install Pueue to `$CARGO_HOME/bin/pueue` (default is `~/.cargo/bin/pueue`)

#### From Source

Pueue is built for the current `stable` Rust version.
It might compile on older versions, but this isn't tested or officially supported.

```bash
git clone git@github.com:Nukesor/pueue
cd pueue
cargo build --release --locked --path ./pueue
```

The final binaries will be located in `target/release/{pueue,pueued}`.

## How to Use it

Check the wiki to [get started](https://github.com/Nukesor/pueue/wiki/Get-started) :).

There are also detailed sections for (hopefully) every important feature:

- [Configuration](https://github.com/Nukesor/pueue/wiki/Configuration)
- [Groups](https://github.com/Nukesor/pueue/wiki/Groups)
- [Miscellaneous](https://github.com/Nukesor/pueue/wiki/Miscellaneous)
- [Connect to remote](https://github.com/Nukesor/pueue/wiki/Connect-to-remote)

On top of that, there is a help option (-h) for all commands.

```text
Interact with the Pueue daemon

Usage: pueue [OPTIONS] [COMMAND]

Commands:
  add            Enqueue a task for execution.
                     There're many different options when scheduling a task.
                     Check the individual option help texts for more information.

                     Furthermore, please remember that scheduled commands are executed via your system shell.
                     This means that the command needs proper shell escaping.
                     The safest way to preserve shell escaping is to surround your command with quotes, for example:
                     pueue add 'ls $HOME && echo "Some string"'
  remove         Remove tasks from the list. Running or paused tasks need to be killed first
  switch         Switches the queue position of two commands. Only works on queued and stashed commands
  stash          Stashed tasks won't be automatically started. You have to enqueue them or start them by hand
  enqueue        Enqueue stashed tasks. They'll be handled normally afterwards
  start          Resume operation of specific tasks or groups of tasks.
                     By default, this resumes the default group and all its tasks.
                     Can also be used force-start specific tasks.
  restart        Restart failed or successful task(s).
                     By default, identical tasks will be created and enqueued, but it's possible to restart in-place.
                     You can also edit a few properties, such as the path and the command, before restarting.
  pause          Either pause running tasks or specific groups of tasks.
                     By default, pauses the default group and all its tasks.
                     A paused queue (group) won't start any new tasks.
  kill           Kill specific running tasks or whole task groups..
                     Kills all tasks of the default group when no ids or a specific group are provided.
  send           Send something to a task. Useful for sending confirmations such as 'y\n'
  edit           Edit the command, path or label of a stashed or queued task.
                     By default only the command is edited.
                     Multiple properties can be added in one go.
  group          Use this to add or remove groups.
                     By default, this will simply display all known groups.
  status         Display the current status of all tasks
  format-status  Accept a list or map of JSON pueue tasks via stdin and display it just like "pueue status".
                     A simple example might look like this:
                     pueue status --json | jq -c '.tasks' | pueue format-status
  log            Display the log output of finished tasks.
                     Only the last few lines will be shown by default.
                     If you want to follow the output of a task, please use the "follow" subcommand.
  follow         Follow the output of a currently running task. This command works like "tail -f"
  wait           Wait until tasks are finished.
                     By default, this will wait for all tasks in the default group to finish.
                     Note: This will also wait for all tasks that aren't somehow 'Done'.
                     Includes: [Paused, Stashed, Locked, Queued, ...]
  clean          Remove all finished tasks from the list
  reset          Kill all tasks, clean up afterwards and reset EVERYTHING!
  shutdown       Remotely shut down the daemon. Should only be used if the daemon isn't started by a service manager
  parallel       Set the amount of allowed parallel tasks
                     By default, adjusts the amount of the default group.
                     No tasks will be stopped, if this is lowered.
                     This limit is only considered when tasks are scheduled.
  completions    Generates shell completion files. This can be ignored during normal operations
  help           Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...         Verbose mode (-v, -vv, -vvv)
      --color <COLOR>      Colorize the output; auto enables color output when connected to a tty [default: auto] [possible values: auto, never, always]
  -c, --config <CONFIG>    Path to a specific pueue config file to use. This ignores all other config files
  -p, --profile <PROFILE>  The name of the profile that should be loaded from your config file
  -h, --help               Print help
  -V, --version            Print version
```

## Design Goals

Pueue is designed to be a convenient helper tool for a single user.

It's supposed to work stand-alone and without any external integration.
The idea is to keep it simple and to prevent feature creep.

Also, **Pueue is considered feature-complete :tada:.**
All features that were planned have been added and only minor improvements, bug-fixes and regular maintenance work will get merged.

For the record, the following features weren't included as they're out of scope:

- Distributed task management/execution.
- Multi-user task management.
- Sophisticated task scheduling for optimal load balancing.
- Tight system integration or integration with external tools.
- Explicit support for scripting.
  If you're adamant about scripting it anyway, take a look at the `pueue-lib` library, which provides proper API calls for `pueued`.
  However, keep in mind that `pueued` is still supposed to be a minimalistic task executor with as little scheduling logic as possible.

There seems to be the need for some project that satisfies all these points mentioned above, but that will be the job of another tool.
I very much encourage forking Pueue and I would love to see forks grow into other cool projects!

## Similar Projects

#### GNU Parallel

A robust and featureful parallel processor with text-based joblog and n-retries. [GNU Parallel](https://www.gnu.org/software/parallel/parallel_tutorial.html) is able to scale to multi-host parallelization and has complex code to have deep integration across different tools and shells, as well as other advanced features. `Pueue` differentiates itself from GNU Parallel by focusing more on visibility across many different long running commands, and creating a central location for commands to be stored, rather than GNU Parallel's focus on chunking a specific task.

#### nq

A very lightweight job queue systems which require no setup, maintenance, supervision, or any long-running processes. \
[Link to project](https://github.com/leahneukirchen/nq)

#### task-spooler

_task spooler_ is a Unix batch system where the tasks spooled run one after the other. \
Links to [ubuntu manpage](http://manpages.ubuntu.com/manpages/xenial/man1/tsp.1.html) and a [fork on Github](https://github.com/xenogenesi/task-spooler).
The original website seems to be down.

## Contributing

Feature requests and pull requests are very much appreciated and welcome!

Anyhow, please talk to me a bit about your ideas before you start hacking!
It's always nice to know what you're working on and I might have a few suggestions or tips :)

Depending on the type of your contribution, you should branch of from either the `main` branch or the `development` branch.

- Bug fixes or critical library updates should branch of `main` and be merged into `main`.
    New patch level releases will be published for this kind of issues.
    Any patches in `main` will also regularly be merged into `development`.
- Everything else, such as new features, refactorings, or breaking changes, should branch of `development` and be merged into `development`.
    Once a new minor or major version has been published, `development` will then be merged into `main`.

There's also the [Architecture Guide](https://github.com/Nukesor/pueue/blob/main/ARCHITECTURE.md), which is supposed to give you a brief overview and introduction to the project.

Copyright &copy; 2019 Arne Beer ([@Nukesor](https://github.com/Nukesor))

