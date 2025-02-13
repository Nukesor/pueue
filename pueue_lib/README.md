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
- Network code. Everything you need to communicate with the daemon.
  - [Request] and [Response] message types.
  - [`send_request`] and [`receive_response`] helper functions.
- A reference [`Client`](client::Client) implementation. This is available with the `client` feature.
  The client also implements [`Client::send_request`] and [`Client::receive_response`].

It also contains helper functions to read local logs.

Pueue-lib is a stand-alone crate, so it can be used by third-party applications to either
manipulate or monitor the daemon or to simply write your own front-end for the daemon.
