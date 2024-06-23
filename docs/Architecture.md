# Architecture Guide

This document is supposed to give you a short introduction to the project. \
It explains the project structure, so you can get a rough overview of the overall architecture.

Feel free to expand this document!

- [Overall Structure](https://github.com/Nukesor/pueue/blob/main/ARCHITECTURE.md#overall-structure)
- [Daemon](https://github.com/Nukesor/pueue/blob/main/ARCHITECTURE.md#daemon)
- [Request Handler](https://github.com/Nukesor/pueue/blob/main/ARCHITECTURE.md#request-handler)
- [TaskHandler](https://github.com/Nukesor/pueue/blob/main/ARCHITECTURE.md#taskhandler)
- [Shared State](https://github.com/Nukesor/pueue/blob/main/ARCHITECTURE.md#shared-state)
- [Code Style](https://github.com/Nukesor/pueue/blob/main/ARCHITECTURE.md#code-style)

## Overall Structure

This project is divided into two modules, the client (`pueue`) and the daemon (`pueued`). \
_Pueue_ also depends on [pueue-lib](https://github.com/nukesor/pueue-lib).
_Pueue-lib_ contains everything that is shared between the daemon and the client.

This includes:

- The protocol used for communicating.
- Settings, since they're parsed by both binaries.
- All data structs, namely `state`, `task` and `message`.
- Helper to interact with task's logs.

## Daemon

The daemon is composed of two main components.

1. Request handling in `pueue/src/daemon/network/`.
   This is the code responsible for communicating with clients.
   In `pueue/src/daemon/network/message_handler/` you can find neatly separated handlers for all of Pueue's subcommands.
2. The TaskHandler in `pueue/src/daemon/task_handler/`.
   It's responsible for everything regarding process interaction.

All information, including process specific information, is stored in the `State` (`pueue-lib/state.rs`) struct. \
Both components share a reference to the State, a `Arc<Mutex<State>>`.
That way we can guarantee a single source of truth and a consistent state.

### Message Handlers

The `pueue/src/daemon/network/socket.rs` module contains the logic for accepting client connections and receiving payloads.
The request accept and handle logic is a single async-await loop run by the main thread.

The payload is then deserialized to `Message` (`pueue-lib/message.rs`) and handled by its respective function.
All functions used for handling these messages can be found in `pueue/src/daemon/network/message_handler`.

### TaskHandler

The TaskHandler is responsible for actually starting and managing system processes. \
It shares the async main thread with the message handlers in a `try_join!` call.

The TaskHandler runs a never ending loop, which checks a few times each second, if

- a new task can be started.
- tasks finished and can be finalized.
- delayed tasks can be enqueued (`-d` flag on `pueue add`)
- A few other things. Check the `TaskHandler::run` function in `pueue/src/daemon/task_handler/mod.rs`.

The TaskHandler is by far the most complex piece of code in this project, but there is also a lot of documentation.

## Shared State

Whenever you're writing some core-logic in Pueue, please make sure to understand how mutexes work.

As a general rule of thumb, the state should only ever be locked in message handler functions and at the top of the TaskHandler's main loop.

This rule allows us to be very conservative with state locking to prevent deadlocks.

## Code Style

This is a result of `tokei ./pueue ./pueue_lib` on commit `84a2d47` at the 2022-12-27.

```
===============================================================================
 Language            Files        Lines         Code     Comments       Blanks
===============================================================================
 JSON                    2          238          238            0            0
 Markdown                2          310            0          192          118
 Pest                    1           69           43           12           14
 TOML                    2          140          112           12           16
 YAML                    1           27           27            0            0
-------------------------------------------------------------------------------
 Rust                  137        12983         9645         1179         2159
 |- Markdown           127         1571            0         1450          121
 (Total)                          14554         9645         2629         2280
===============================================================================
 Total                 145        13767        10065         1395         2307
===============================================================================
```

### Format and Clippy

`cargo format` and `cargo clean && cargo clippy` should never return any warnings on the current stable Rust version!

PR's are automatically checked for these two and won't be accepted unless everything looks fine.

### Comments

1. All functions must have a doc block.
2. All non-trivial structs must have a doc block.
3. Rather too many inline comments than too few.
4. Non-trivial code should be well documented!

In general, please add a lot of comments. It makes maintenance, collaboration and reviews MUCH easier.
