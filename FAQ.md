# Frequently Asked Questions


## The Project Does Not Compile:
Do you have Rust version `>=1.39` installed?
If that's not the case, there should also be warning at the very top of your `cargo build` output.

If you have Rust version `>=1.39`, please file a bug report.


## A Command Doesn't Behave Like Expected

**First thing to do:** Try to run the command without adding it to pueue. If this fails, it's not a problem with Pueue.

**Second thing** to do when debugging any problems with running/failing processes, is to look at the process output:

This can be done via `pueue log $task_id` for finished processes or `pueue show $task_id` for running processes.\
You can also get a live view of the output with `pueue show -f $task_id` for `stdout` and `-e` for `stderr`.


### The Command Formatting Seems To Be Broken:

Pueue takes your input and uses it exactly as is to create a new `bash -c $command` in the background.\
If your command contains spaces or characters that need escaping, you might need to encapsulate it into a string:

```
    pueue add -- ls -al "/tmp/this\ is\ a\ test\ directory"
```

Without quotes, the character escaping won't be transferred to the `bash -c $command`, as it's already removed by calling it from the current shell.


### A Process Waits For Input:

Sometimes some process waits for input. For instance, a package manager may wait for confirmation (`y/n`).\

In this case you can send the desired input to the process via:

```
pueue send "y
"
```

This can be also be avoided by issuing the command with something like a `-y` flag (if it allows something like this),
you see that a process waits for input, you can 


### My Shell Aliases Don't Work:

This is a known problem. 
Since pueue calls a new shell session without any parameters, existing `.bashrc` won't be read.

However, reading `.bashrc` files turns out to be problematic as well.
Pueue might add a feature for custom shell commands somehwere in the future, but this isn't working for now.


# What's The Advantage Over Using A Terminal Multiplexer

- The ability to queue commands and not start them all at once
- You can specify how many parallel tasks you want
- Easy pausing/resuming of tasks
- A very pretty printed table with a good overview
- You don't need to attach to a tmux session or anything similar, if operating on a server and not wanting to have an open session all the time.

Additionally the long term plan is:

- to add groups (You can specify the amount of parallel tasks per groups)
- to allow remote connection to servers. No need to ssh onto a server, you can manipulate and check the status directly on your local machine.

A lot of its functionality is convenience. Using your shell's tools is possible, but IMO, having something that's specifically designed for this task is more efficient and definitely more convenient.


For example, one of my very regular use cases is Movie encoding. In this case I want:

- at most two parallel encodes (otherwise performance degrades).
- them to be processed one after another
- to see at first glance, whether a command fails
- an easy way to look at output
- everything to be in a uniform interface
- it to look pretty and clear
- being able to pause/resume everything in case I need to run something with full power on my server right NOW
- I probably forgot a few things

I used tmux for this stuff all the time before writing Pueue.
However, after using it for a really long time, it just kept feeling annoying and inconvenient.
Up to the point I couldn't bare it any longer and decided to write something that's better suited for such scenarios.

Additionally, I already used Pueue as a TaskManager in combination with some other tools (It has a simple JSON API via TCP).
Those tools need to be adjusted for the Rust rewrite, but back then it worked like a charm.
