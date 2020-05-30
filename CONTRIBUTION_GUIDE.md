# Hi!

This document is supposed to give you a short introduction to the project.

It's purpose is to explain the project structure, so you understand where you can find the parts of code you're looking for.


# Structure

This project is divided into three modules. `client`, `daemon` and `shared`.
`client` and `daemon` contain everything that's specific to the respective binaries of `pueue` and `pueued`.

`shared` however contains everything that's used by both sides.
This includes:

- The protocol used for communicating
- Settings, since they're parsed by both binaries
- All data objects, namely `state`, `task` and `message`
- Logging


# Daemon

The daemon is composed of two main components.
Both components own a `Arc<Mutex<State>>`, so we can guarantee a single source of truth.

It's also important to know, that there's a `mpsc` channel, with the TaskHandler being the consumer.
This allows to notify the TaskHandler of any special tasks that need to be done.

**1. The TcpListener**

The `daemon.socket` module contains the logic for accepting and low-level handle client connections.

These are then interpreted and handled by a respective function.
All functions used for handling these messages can be found in `daemon.instructions`.

Many messages can be handled directly, by modifying the state.
However, sometimes we need to notify the TaskHandler, in case we need something special.

A few examples:
- Instant starting of tasks
- Pausing/resuming of tasks
- Resetting the daemon


**2. The TaskHandler**

The TaskHandler is responsible for actually starting and handling the processes as specified in a `Task`.
It's further important to note, that it runs in it's own thread.

Handling tasks include:
- Starting/killing tasks
- Pausing/resuming tasks
- Handling finished tasks
- Handling dependencies
- Handling scheduled tasks

There's a lot of rather complicated code in this file.

Whenever you're editing in here, please make sure to be conservative with your `state.lock()` calls.
`state.lock()` blocks everything else!
You should always lock as late as possible and **only** use it if absolutely necessary.
