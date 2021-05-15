# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.13.0] - 

## Changed

- Use `serde_cbor` instead of `bincode` to allow protocol backward compatibility between versions
- Use the next id that's available. This results in ids being reused, on `pueue clean` or `pueue remove` of the last tasks in a queue.
- Paths are now accessed via functions by [dadav](https://github.com/dadav) for [Pueue #191](https://github.com/Nukesor/pueue/issues/191)
- Remove `full` flag from TaskLogRequestMessage.
- Automatically create `$pueue_directory/certs` directory on `create_certificates` if it doesn't exist yet.
- Remove `require_config` flag from `Settings::read`, since it's implicitely `true`.
- Rename `Settings::new`, to `Settings::read_with_defaults`.

## Added

- `~` is now respected in configuration paths by [dadav](https://github.com/dadav) for [Pueue #191](https://github.com/Nukesor/pueue/issues/191).
- New function `read_last_log_file_lines` for [#196](https://github.com/Nukesor/pueue/issues/196).
- Add `callback_log_lines` setting for Daemon, specifying the amount of lines returned to the callback. [#196](https://github.com/Nukesor/pueue/issues/196).
- Support for other `apple` platforms by [althiometer](https://github.com/althiometer)

## Fixed

- Only try to remove log files, if they actually exist.

## [0.12.2] - 30-03-2021

## Changed

- Clippy adjustment: Transform `&PathBuf` to `&Path` in function parameter types.
    This should be reverse-compatible, since `&PathBuf` dereferences to `&Path`.

## [0.12.1] - 09-02-2021

### Added

- `dark_mode` client configuration flag by [Mephistophiles](https://github.com/Mephistophiles)

## [0.12.0] - 04-02-2021

Moved into a stand-alone repository for better maintainability.

### Changed

- Change the packet size from 1.5 Kbyte to 1.4 Kbyte to prevent packet splitting on smaller MTUs.
- Add LOTS of documentation.
- Hide modules that aren't intended for public use.
- Rename `GenericListener` to `Listener` and vice versa.

### Removed

- Remove unused `group_or_default` function.

## [0.11.2] - 01-02-2021

### Changed

- Use `127.0.0.1` instead of `localhost` as default host.
    This prevents any unforseen consequences if somebody deletes the default `localhost` entry from their `/etc/hosts` file.

## [0.11.0] - 18-01-2020

### Fixed

- Moved into a stand-alone repository for better maintainability.
- Don't parse config path, if it's a directory.
- Error with "Couldn't find config at path {:?}" when passing a directory via `--config`.
- Fixed missing newline between tasks in `log` output.
