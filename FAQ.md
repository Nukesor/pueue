# Frequently Asked Questions


## The Project Does Not Compile:
Do you have Rust version `>=1.39` installed?
If that's not the case, there should also be warning at the very top of your `cargo build` output.

If you have Rust version `>=1.39`, please file a bug report.


## A Command Doesn't Behave Like Expected

Pueue only executes specified commands.
First thing to do when debugging any problems with running/failing processes, is to look at the process output:

This can be done via `pueue log $task_id` for finished processes or `pueue show $task_id` for running processes.\
You can also get a live view of the output with `pueue show -f $task_id` for `stdout` and `-e` for `stderr`.

### The Command Formatting Seems To Be Broken:

Pueue takes your input and uses it exactly as is to create a new `bash -c $command` in the background.\
If your command contains spaces or characters that need escaping, you might need to encapsulate it into a string:

```
    pueue add "ls -al /tmp/this\ is\ a\ test\ directory"
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
