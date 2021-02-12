# Architecture Guide

This document is supposed to give you a short introduction to the project. \
It explains the project structure, so you can get a rough overview of the overall architecture.

Feel free to expand this document!

- [Overall Structure](https://github.com/Nukesor/pueue/blob/master/ARCHITECTURE.md#overall-structure)
- [Daemon](https://github.com/Nukesor/pueue/blob/master/ARCHITECTURE.md#daemon)
- [Request Handler](https://github.com/Nukesor/pueue/blob/master/ARCHITECTURE.md#request-handler)
- [TaskHandler](https://github.com/Nukesor/pueue/blob/master/ARCHITECTURE.md#taskhandler)

## Overall Structure

This project is divided into two modules, the client (`pueue`) and the daemon (`pueued`). \
_Pueue_ also depends on [pueue-lib](https://github.com/nukesor/pueue-lib).
_Pueue-lib_ contains everything that is shared between the daemon and the client.

This includes:

- The protocol used for communicating
- Settings, since they're parsed by both binaries
- All data structs, namely `state`, `task` and `message`
- Helper to interact with task's logs

## Daemon

The daemon is composed of two main components.

1. Request handling in `daemon/network/socket.rs`. It's used for communication with clients.
2. The TaskHandler in `daemon/task_handler.rs`. It's responsible for everything regarding process interaction.

All information that's not process specific, is stored in the `State` (`pueue-lib/state.rs`) struct. \
Both components share a reference to the State, a `Arc<Mutex<State>>`.
That way we can guarantee a single source of truth and a consistent state.

It's also important to know, that there's a `mpsc` channel. \
This channel is used to send on-demand messages from the request handler to the the TaskHandler.
This includes stuff like "Start/Pause/Kill this task right now" or "Reset everything".

### Request handling

The `daemon/network/socket.rs` module contains the logic for accepting client connections and receiving payloads.
The whole request accept-handle logic is a single async-await loop run by the main thread.

The payload is then deserialized to `Message` (`pueue-lib/message.rs`) and handled by a respective function.
All functions used for handling these messages can be found in `daemon/network/message_handler`.

Many messages can be instantly handled by simply modifying or reading the state. \ 
However, sometimes the TaskHandler has to be notified, if something involves modifying actual system processes (start/pause/kill tasks).
That's when the `mpsc` channel to the TaskHandler comes into play.

### TaskHandler

The TaskHandler is responsible for actually starting and managing system processes. \
It's further important to note, that it runs in it's own thread.

The TaskHandler runs a never ending loop, which checks a few times each second, if

- there are new instructions in the `mpsc` channel.
- a new task can be started.
- tasks finished and can be finalized.
- delayed tasks can be enqueued (`-d` flag on `pueue add`)


The TaskHandler is by far the most complex piece of code in this project, but there is also a lot of documentation.

## Shared State

Whenever you're writing some core-logic in Pueue , please make sure to understand how mutexes work.

Try to be conservative with your `state.lock()` calls, since this also blocks the request handler!
Only use the state, if you absolutely have to.

At the same time, you should also lock early enough to prevent inconsistent states. \
Working with mutexes is always a little bit tricky.

## Code Style

This is a result of `tokei ./pueue/{client,daemon} ./pueue-lib/src` on commit `662f560` at the 2021-02-21.

```
===============================================================================
 Language            Files        Lines         Code     Comments       Blanks
===============================================================================
 Rust                   69         6396         4949          492          955
 |- Markdown            54          695            0          657           38
 (Total)                           7091         4949         1149          993
===============================================================================
 Total                  69         6396         4949          492          955
===============================================================================
```

### Format and Clippy

`cargo format` and `cargo clean && cargo clippy` should never return any warnings on the current stable Rust version!

PR's are automatically checked for these two and won't be accepted unless everything looks fine.

### Comments

1. All functions have to have a doc block.
2. All non-trivial structs have to have doc block.
3. Rather too many inline comments than too few.
4. Everything with non-trivial complexity should be well documented!

In general, please add a lot of comments. It makes maintenance, collaboration and reviews MUCH easier.
