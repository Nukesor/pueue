# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres **somewhat** to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The concept of SemVer is applied to the daemon/client API, but not the library API itself.

## [0.28.1] - 2025-02-17

### Added

- Extracted `pueue`'s `Client` implementation into `pueue_lib`.
  It uses `color-eyre` for generic error handling, which could be improved in the future.
  Otherwise, it serves as a convenient entry point to implement a client.

## [0.28.0] - 2025-02-11

### Changed

- **Breaking**: Split the `Message` enum into a `Response` and `Request` enum. Requests are sent **to** the daemon, responses are sent **from** the daemon.
- **Breaking**: Move all process handling related logic out of `pueue_lib`
- **Breaking**: Remove all process related fields and helper functions from `State`

## [0.27.0] - 2024-12-01

### Added

- Introduce re-usable `Children` Process representation.

### Removed

- Remove unused `reset_task_log_directory`
- Add `EnvMessage` to set/unset environment variables on tasks.
- `netbsd` support

### Changed

- Rework how `reset` works. `Reset` now works on a per-group basis.
  Full reset simply means that all groups are to be reset.
- Move **all** state into the `State` struct.
  This prevents concurrency issues and fixes many issues.
  The new runtime-dependant fields aren't serialized.
- Rework `TaskState` representation enum to include all runtime-dependant fields.
  As a result, several fields have been removed from `Task.
- Make `Stash` Message payload a struct.
- Make `EditRestore`, `EditRequest` Message payload a vector of task ids.
- Make `Edit` Message payload a vector the new `EditableTask`.
- Change editing structs so that all values are always set (no more optionals
- Change `ResetMessage` to use new `ResetTarget` enum.

### Fix

- Unix socket permission handling

## [0.26.0] - 2024-03-22

### Added

- Added `priority` field on `EditResponseMessage`, `EditMessage` and `TaskToRestart`.

## [0.25.1] - 2024-01-04

### Changed

- Bump dependencies. Most notably `ring` from 0.16 to 0.17 to add riscv64 support [#484](https://github.com/Nukesor/pueue/issues/484).

## [0.25.0] - 2023-10-21

### Added

- `Task::is_stashed()`
- `Default` impl for `Setting`
- Support the `PUEUE_CONFIG_PATH` environment variable in addition to the `--config` option. [#464](https://github.com/Nukesor/pueue/issues/464)
- Experimental: Allow configuration of the shell command that executes task commands. [#454](https://github.com/Nukesor/pueue/issues/454)
- Experimental: Allow injection of hard coded environment variables via config file. [#454](https://github.com/Nukesor/pueue/issues/454)

### Changed

- The `filter_tasks_*` functions of `State` have been refactored to be more clear and ergonomic to use.

## [0.24.0] - 2023-06-13

### Added

- New setting `daemon.shell_command` to configure how the command shall be executed.
- New setting `daemon.env_vars` to inject hard coded environment variables into the process.

### Changed

- Refactor `State::filter_*` functions to return proper type.

## [0.23.0] - 2023-06-13

### Added

- Add `priority` field to `Task`
- Remove `tempdir` dependency

## [0.22.0]

This version was skipped due to a error during release :).

## [0.21.3] - 2023-02-12

### Changed

- Switched the test suite on MacOS to use the new `libproc::processes::pids_by_type()` API to enumerate PIDs in a program group, removing the need to depend on the unmaintained darwing-librproc library. [#409](https://github.com/Nukesor/pueue/issues/409).

## [0.21.2] - 2023-02-08

### Fix

- Point to a new patched fork of `darwin-libproc`, as the original has been deleted.
  This fixes the development builts for pueue on Apple platforms.

## [0.21.0] - 2022-12-12

### Breaking Changes

- Tasks are now started in a process group, with signals (including SIGTERM) optionally sent to the whole group. [#372](https://github.com/Nukesor/pueue/issues/372)
- Renamed `TasksToRestart` to `TaskToRestart`.
- Make `TaskToRestart::path` and `TaskToRestart::command` optional.
- Make `EditMessage::path` and `EditMessage::command` optional.
- The `children` flag has been removed for the `Start`-,`Pause`-,`Kill`- and `ResetMessage`.
- No longer support TLS 1.2 certificates, only accept version 1.3.
  All generated certificates were 1.3 anyway, so there shouldn't be any breakage, except users created their own certs.

### Added

- Added `Settings.shared.alias_file`, which can be used to specify the location of the `pueue_aliases.yml`.
- Added functionality to edit a task's label [#354](https://github.com/Nukesor/pueue/issues/354).
  - `TaskToRestart.label`
  - `TaskToRestart.delete_label`
  - `EditMessage.label`
  - `EditMessage.delete_label`
- Added `Task.enqueued_at` and `Task.created_at` metadata fields [#356](https://github.com/Nukesor/pueue/issues/356).

### Changed

- The module structure of the platform specific networking code has been streamlined.
- The process handling code has been moved from the daemon to `pueue_lib`. See [#336](https://github.com/Nukesor/pueue/issues/336).
  The reason for this is, that the client will need some of these process handling capabilitites to spawn shell commands when editing tasks.

## [0.20.0] - 2022-07-21

### Added

- `Message::Close` which indicates the client that everything is done and the connection is being closed.

### Removed

- Breaking change: Backward compatibility logic for the old group structure in the main state.
- Breaking change:
  The `State` no longer owns a copy of the current settings.
  This became possible due to the group configuration no longer being part of the configuration file.

### Fixed

- The networking logic wasn't able to handle rapid successiv messages until now.
  If two messages were sent in quick succession, the client would receive both messages in one go.
  The reason for this was simply that the receiving buffer was always of a size of 1400 Bytes, even if the actual payload was much smaller.
  This wasn't a problem until now as there was no scenario where two messages were send immediately one after another.

## [0.19.6] - unreleased

### Added

- Docs on how pueue's communication protocol looks like [#308](https://github.com/Nukesor/pueue/issues/308).

## [0.19.5] - 2022-03-22

### Added

- Settings option to configure pid path

## [0.19.4] - 2022-03-12

### Added

- New `Error::IoPathError` which provides better error messages for path related IoErrors.
- `Error::RawIoError` for all generic IoErrors that already have some context.

### Changed

- More info in `Error::IoError` for better context on some IoErrors.

### Removed

- `Error::LogWrite` in favor of the new `IoPathError`.
- `Error::LogRead` in favor of the new `IoPathError`.
- `Error::FileNotFound` in favor of the new `IoPathError`.

## [0.19.3] - 2022-02-18

### Changed

- Use PathBuf in all messages and structs for paths.

## [0.19.2] - 2022-02-07

### Changed

- Make most configuration sections optional.

## [0.19.1] - 2022-01-31

- Update some dependencies for library better stability

## [0.19.0] - 2022-01-30

### Added

- Add optional `group` field to CleanMessage.
- Add optional `parallel_tasks` field to Group create message.
- Introduced a `Group` struct which is used to store information about groups in the `State`.
- Added the `shared.runtime_directory` config variable for any runtime related files, such as sockets.
- `XDG_CONFIG_HOME` is respected for Pueue's config directory [#243](https://github.com/Nukesor/pueue/issues/243).
- `XDG_DATA_HOME` is used if the `pueue_directory` config isn't explicitly set [#243](https://github.com/Nukesor/pueue/issues/243).
- `XDG_RUNTIME_DIR` is used if the new `runtime_directory` config isn't explicitly set [#243](https://github.com/Nukesor/pueue/issues/243).
- Add `lines` to `LogRequestMessage` [#270](https://github.com/Nukesor/pueue/issues/270).
- Add a new message type `Message::EditRestore` which is used to notify the daemon of a failed editing process.

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
- Support for other `apple` platforms.
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
