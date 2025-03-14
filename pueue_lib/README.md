# Pueue-lib

[![Test Build](https://github.com/Nukesor/pueue/actions/workflows/test.yml/badge.svg)](https://github.com/Nukesor/pueue/actions/workflows/test.yml)
[![Crates.io](https://img.shields.io/crates/v/pueue-lib)](https://crates.io/crates/pueue-lib)
[![docs](https://docs.rs/pueue-lib/badge.svg)](https://docs.rs/pueue-lib/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This is the shared library used by the [Pueue](https://github.com/nukesor/pueue/) client and daemon.

It contains everything you need to communicate with the daemon:

- The [State], which represents the current state of the daemon as exposed to clients.
- The [Task], [TaskResult] and [TaskStatus]
- The [Settings] used by both clients and the daemon.
- `async` and `blocking` Network code. Everything you need to communicate with the daemon.
  - [Request] and [Response] message types.
  - [`network::send_request`] and [`network::receive_response`] helper functions.
- A reference [`Client`](Client) implementation. This is available with the `client` feature.
  The client also implements [`Client::send_request`] and [`Client::receive_response`].

It also contains helper functions to read local logs.

Pueue-lib is a stand-alone crate, so it can be used by third-party applications to either
manipulate or monitor the daemon or to simply write your own front-end for the daemon.

## Features

For a minimal setup, disable default features and enable `client` and `network` or `network_blocking`.

- `client` Adds a [`Client`] and/or `BlockingClient` implementation, depending on whether `network` and/or `network_blocking` features are active.
- `network` adds async network and protocol functions.
- `network_blocking` adds blocking `std` network and protocol functions.
- `log` adds functions for reading pueue's log files on the local machine.
- `settings` [`Settings`] struct used by both the daemon and client.
