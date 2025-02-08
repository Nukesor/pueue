# Architecture Guide

This document is supposed to give you a short introduction to the project. \
It explains the project structure, so you can get a rough overview of the overall architecture.

Feel free to expand this document!

- [Overall Structure](#overall-structure)
- [Daemon](#Daemon)
  - [Main loops](#main-loops)
    - [Task handler main loop](#task-handler-main-loop)
    - [Message handler main loop](#message-handler-main-loop)
  - [Message Handlers](#message-handlers)
  - [Process Handlers](#process-handlers)
- [Shared State](#shared-state)
- [Code Style](#code-style)

## Overall Structure

This project is divided into two modules, the client (`pueue`) and the daemon (`pueued`). \
Both depends on [pueue-lib](https://github.com/nukesor/pueue-lib).
_pueue-lib_ contains everything that is shared between the daemon and the client.

This includes:

- The protocol used for communicating.
- Settings, since they're parsed by both binaries.
- All data structs, namely `state`, `task` and `message`.
- Helper to interact with task's logs.

## Daemon

The daemon is composed of two main components and a bunch of helper functions.

1. Request handling in [`daemon::network`].
   This is the code responsible for communicating with clients.
   In [`daemon::network::message_handler`] you can find neatly separated handlers for all of Pueue's subcommands.
2. Process handling is located in [`daemon::process_handler`].
   Each file contains functions to handle a specific type of process related operation.

All information, including process specific information, is stored in the [`State`] struct. \
Both components share a reference to the State, via a [`SharedState`] handle, which is effectively a `Arc<Mutex<State>>`.
That way we can guarantee a single source of truth and a consistent state at all times.

### Main loops

The daemon has two main loops. One for handling client requests and one for handling task's processes.
Both run in parallel in the same multi-threaded tokio async runtime via a `try_join!` call.

#### Task handler main loop

The task handling main loop is located in [`daemon::task_handlers::run`]
It takes care of the actual "daemon" and scheduling logic.

- Scheduling/starting of new tasks when a new slot is available
- Handling finished tasks and cleaning up processes.
- Enqueueing delayed tasks that reached their `enqueue_at` date.
- Callback process handling.
- Task dependency checks (mark tasks as failed if their dependencies failed).
- Reset logic
- Shutdown logic

### Message handler main loop

The message handler main loop is the `accept_incoming` function located in [`daemon::network::socket`].

To give a rough overview of what happens in here:

- Listen on the daemon's socket for incoming Unix/TCP connections
- Accept new connection
- For each new connection, spawn a new tokio task that calls the `handle_incoming` function.
- Performs authorization (secret & certificate)
- If successful, send confirmation, which is also the daemon's version.
- Receive the incoming message and deserialize it
- Handle the message via the [`handle_message`] function.
- Return the response

### Message Handlers

All functions used for handling client messages can be found in [`daemon::network::message_handler`].

Message handling functions have a [`SharedState`] handle and may call [`daemon::process_handler`] functions
directly to immediately execute tasks such as starting, stopping or killing processes.

### Process Handlers

The [`daemon::process_handler`] functions are used to actually start and manage system processes. \

These functions are by far the most complex piece of code in this project, but there is also a lot of documentation for each individual function, so go and check them out :).

## Shared State

Whenever you're writing some core-logic in Pueue, please make sure to understand how mutexes work.

As a general rule of thumb, the [`SharedState`] should only ever be locked in message handler functions and at the start of any process handling functionality.
Always make sure that you lock the state for a given "unit of work".
This rule allows us to be very conservative with state locking to prevent deadlocks.

## Code Style

This is a result of `tokei ./pueue ./pueue_lib` on commit `1db4116` at the 2025-02-08.

```
===============================================================================
 Language            Files        Lines         Code     Comments       Blanks
===============================================================================
 JSON                    2          250          250            0            0
 Markdown                3          404            0          252          152
 Pest                    1           74           46           13           15
 TOML                    2          161          136           12           13
 YAML                    1           27           27            0            0
-------------------------------------------------------------------------------
 Rust                  150        16067        11971         1492         2604
 |- Markdown           137         1840            0         1660          180
 (Total)                          17907        11971         3152         2784
===============================================================================
 Total                 159        16983        12430         1769         2784
===============================================================================
```

### Format and Clippy

`cargo format` and `cargo clean && cargo clippy` should never return any warnings on the current stable Rust version!

PR's are automatically checked for these two and won't be accepted unless everything looks fine.

### Comments

1. All functions must have a doc block.
2. All non-trivial structs must have a doc block.
3. Write rather too many inline comments than too few.
4. Non-trivial code should be well documented!

In general, please add a lot of comments. It makes maintenance, collaboration and reviews MUCH easier.

[`Message`]: `https://docs.rs/pueue-lib/latest/pueue_lib/network/message/enum.Message.html`
[`SharedState`]: https://docs.rs/pueue-lib/latest/pueue_lib/state/type.SharedState.html
[`State`]: https://docs.rs/pueue-lib/latest/pueue_lib/state/struct.State.html
[`daemon::network::message_handler`]: https://github.com/Nukesor/pueue/blob/main/pueue/src/daemon/network/message_handler
[`daemon::network::socket`]: https://github.com/Nukesor/pueue/blob/main/pueue/src/daemon/network/socket.rs
[`daemon::network`]: https://github.com/Nukesor/pueue/blob/main/pueue/src/daemon/network
[`daemon::process_handler`]: https://github.com/Nukesor/pueue/tree/main/pueue/src/daemon/process_handler
[`daemon::task_handlers::run`]: https://github.com/Nukesor/pueue/blob/main/pueue/src/daemon/task_handler.rs
[`handle_message`]: https://github.com/Nukesor/pueue/blob/main/pueue/src/daemon/network/message_handler/mod.rs
