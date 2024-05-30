# Pueue-lib

[![Test Build](https://github.com/Nukesor/pueue/actions/workflows/test.yml/badge.svg)](https://github.com/Nukesor/pueue/actions/workflows/test.yml)
[![Crates.io](https://img.shields.io/crates/v/pueue-lib)](https://crates.io/crates/pueue-lib)
[![docs](https://docs.rs/pueue-lib/badge.svg)](https://docs.rs/pueue-lib/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This is the shared library used by the [Pueue](https://github.com/nukesor/pueue/) client and daemon.

It contains common components such as:

- Everything about the [Task](task::Task), [TaskResult](task::TaskResult) etc.
- The [State](state::State), which represents the current state of the daemon.
- Network code. Everything you need to communicate with the daemon.
- Other helper code and structs.

Pueue-lib is a stand-alone crate, so it can be used by third-party applications to either
manipulate or monitor the daemon or to simply write your own front-end for the daemon.
