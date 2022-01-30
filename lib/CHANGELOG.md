# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.19.0] - 30-01-2022

### Added

- Add optional `group` field to CleanMessage.
- Add optional `parallel_tasks` field to Group create message.
- Introduced a `Group` struct, which is used to store information about groups in the `State`.
- Added the `shared.runtime_directory` config variable for any runtime related files, such as sockets.
- `XDG_CONFIG_HOME` is respected for Pueue's config directory [#243](https://github.com/Nukesor/pueue/issues/243).
- `XDG_DATA_HOME` is used if the `pueue_directory` config isn't explicitly set [#243](https://github.com/Nukesor/pueue/issues/243).
- `XDG_RUNTIME_DIR` is used if the new `runtime_directory` config isn't explicitly set [#243](https://github.com/Nukesor/pueue/issues/243).
- Add `lines` to `LogRequestMessage` [#270](https://github.com/Nukesor/pueue/issues/270).
- Add a new message type `Message::EditRestore`, which is used to notify the daemon of a failed editing process.

### Removed

- Remove the `settings.daemon.default_parallel_tasks` setting, as it doesn't have any effect.

### Changed

- Switch from `async-std` to tokio.
- Update to rustls 0.20
- **Breaking:** Logs are now no longer split into two files, for stderr and stdout respectively, but rather a single file for both.
- **Breaking:** The unix socket is now located in the `runtime_directory` by default [#243](https://github.com/Nukesor/pueue/issues/243).
- **Breaking:** `Shared::pueue_directory` changed from `PathBuf` to `Option<PathBuf>`.
- **Breaking:** `Settings::read_with_defaults` no longer a boolean as first parameter.
    Instead, it returns a tuple of `(Settings, bool)` with the boolean indicating whether a config file has been found.
- **Breaking:** The type of `State.group` changed from `BTreeMap<String, GroupStatus>` to the new `BTreeMap<String, Group>` struct.
- **Breaking:** The `GroupResponseMessage` now also uses the new `Group` struct.

## [0.18.1] - 2021-09-15

### Added

- Add the `PUEUE_DEFAULT_GROUP` constant, which provides a consistent way of working with the `"default"` group.

### Fix

- Always insert the "default" group into `settings.daemon.group` on read.

## [0.18.0] - 2021-07-27

### Change

- Make `GroupMessage` an enum to prevent impossible states.
- Introduce `TaskSelection` enum to prevent impossible states in Kill-/Start-/PauseMessage structs.

## [0.17.2] - 2021-07-09

### Fix

- Fix default for `client.restart_in_place` to previous default.

## [0.17.1] - 2021-07-08

### Fix

- Add missing config default for `client.status_time_format` and `client.status_datetime_format`

## [0.17.0] - 2021-07-08

### Added

- Add config option to restart tasks with `in_place` by default.

### Changed

Remove defaults for backward compatibility.
We broke that in the last version anyway, so we can use this opportunity and clean up a little.

## [0.16.0] - 2021-07-05

This release aims to remove non-generic logic from `State`, that should be moved to the `Pueue` project.

### Added

- Add config option for datetime/time formatting in pueue status.

### Changed

- Make `State::config_path` public.

### Removed

- `State::handle_task_failure`
- `State::is_task_removable`
- `State::task_ids_in_group_with_stati` in favor of `State::filter_tasks_of_group`
- `State::save`, `State::backup`, `State::restore` and all related functions.
- State related errors from the custom `Error` type.

## [0.15.0] - 2021-07-03

Several non-backward compatible breaking API changes to prevent impossible states.

### Changed

- Remove `tasks_of_group_in_statuses` and `tasks_in_statuses` in favor of generic filter functions `filter_tasks_of_group` and `filter_tasks`.
- Move `TaskResult` into `TaskStatus::Done(TaskResult)` to prevent impossible states.
- Move `enqueue_at` into `TaskStatus::Stashed{enqueue_at: Option<DateTime<Local>>}` for better contextual data structure.

## [0.14.1] - 2021-06-21

### Added
- Messages now have PartialEq for better testability

## [0.14.0] - 2021-06-15

### Changed

- Add `ShutdownType` to `DaemonShutdownMessage`

## [0.13.1] - 2021-06-04

- Add `State::tasks_of_group_in_statuses`

## [0.13.0] - 2021-05-28

### Changed

- Use `serde_cbor` instead of `bincode` to allow protocol backward compatibility between versions
- Use the next id that's available. This results in ids being reused, on `pueue clean` or `pueue remove` of the last tasks in a queue.
- Paths are now accessed via functions by [dadav](https://github.com/dadav) for [Pueue #191](https://github.com/Nukesor/pueue/issues/191)
- Remove `full` flag from TaskLogRequestMessage.
- Automatically create `$pueue_directory/certs` directory on `create_certificates` if it doesn't exist yet.
- Remove `require_config` flag from `Settings::read`, since it's implicitely `true`.
- Rename `Settings::new`, to `Settings::read_with_defaults`.
- Return errors via `Result` in `State` functions with io.
- Don't write the State on every change. Users have to call `state::save()` manually from now on.

### Added

- `~` is now respected in configuration paths by [dadav](https://github.com/dadav) for [Pueue #191](https://github.com/Nukesor/pueue/issues/191).
- New function `read_last_log_file_lines` for [#196](https://github.com/Nukesor/pueue/issues/196).
- Add `callback_log_lines` setting for Daemon, specifying the amount of lines returned to the callback. [#196](https://github.com/Nukesor/pueue/issues/196).
- Support for other `apple` platforms by [althiometer](https://github.com/althiometer)
- Added backward compatibility tests for v0.12.2 state.
- Added SignalMessage and Signal enum for a list of all supported Unix signals.

### Fixed

- Only try to remove log files, if they actually exist.

## [0.12.2] - 30-03-2021

### Changed

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
