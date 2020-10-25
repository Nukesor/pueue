## Start the Daemon
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

### Adding Commands

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

### Manipulate multiple tasks at once

Most commands can be executed on multiple tasks at once.
For instance, you can look at specific logs like this:\
`pueue log 0 1 2 3 15 19`.

This also works with your shell's range parameter, e.g. `pueue log {0..3} 15 19`.

**Pitfalls:**

To avoid common pitfalls, please read the [FAQ Section](https://github.com/Nukesor/pueue/blob/master/FAQ.md).