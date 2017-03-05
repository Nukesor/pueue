# Change Log

All notable changes to this project will be documented in this file.


## [0.8.0]

### Added
- Allow switching of `stashed` entries in queue.

### Changed
- `remove`, `restart`, `stash`, `enqueue` commands can receive multiple keys instead of a single key.
- `log`, `pause`, `start`, `kill`, `stop` commands don't have a `--key` parameter anymore. They can now receive a list of keys without providing a flag i.e. `pueue start 0 1` instead of `pueue start -k 0 && pueue start -k 1`. The default behavior if no key is provided stays the same for all commands.
- Daemon API now requires a `keys` parameter where `type(keys) == list` for the commands listed above.

### Fixed
- Removed `key` parameter from `send` command.
- Wrong daemon response for `kill` command.
- `stop` or `kill` sends the signal to all processes spawned by the shell process. This bug affected all command strings which caused the subprocess to spawn a `/bin/sh -c {command}` process.
