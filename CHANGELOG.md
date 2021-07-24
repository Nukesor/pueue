# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 

A lot of things happened during this release.
Even though quite a few new features were added, the main effort went into increasing stability and inter-version compatibility.

The goal of this release is to push the code quality, error handling, test coverage and stability to a level that justifies a v1.0 release. \
Since this project follows semantic versioning, this includes no breaking changes and backward compatibility on version upgrades. \
It also includes, that I'm quite certain there are no critical bugs in the project and that all important and planned features have been implemented.

Unless some critical issues pop up, this can be seen as a finished version of the project!

**Disclaimer:** This project is mainly developed for Linux.
Windows and MacOS/Apple platforms are partially supported, but this is a community effort.
Thereby, v1.0 might be misleading for those. \
I hope you understand, that I cannot wait a few years for someone to implement missing features for these platforms.
I want this project to move forward.

### Added

- The last lines of `stderr` and `stdout` are now available in the callback command. [#196](https://github.com/Nukesor/pueue/issues/196).
- Add `callback_log_lines` setting for the daemon, specifying the amount of lines returned to the callback. [#196](https://github.com/Nukesor/pueue/issues/196).
- `~` is respected in configuration paths by [dadav](https://github.com/dadav) for [#191](https://github.com/Nukesor/pueue/issues/191).
- Support for other `apple` platforms. New build artifacts for `ios-aarch64`.
- Use `pueue kill --signal SigTerm` to send Unix signals directly to Pueue's processes. [#202](https://github.com/Nukesor/pueue/issues/202)
- Add a PID file to `$pueue_directory/pueue.pid`, which will be used to check whether there's an already running daemon.
- `--failed-in-group [group_name]` for `restart`. That way you can restart all failed tasks of a specific group [#211](https://github.com/Nukesor/pueue/issues/211)
- Options to configure the time and datetime format in `pueue status` for [##212](https://github.com/Nukesor/pueue/issues/212).
- Option to use the `--in-place` flag on `restart` by default.

### Changed

- Use the next available id instead of constantly increasing id's.
    This results in ids being reused, on `pueue clean` or `pueue remove` of the last tasks in a queue.
- Show the date in `pueue status` for the `start` and `end` fields, if the task didn't start today.
- Backward compatible protocol for stable version changes with `serde_cbor`.
- Detection of old daemon versions on update.
- Overall better debug messages.
- Crash hard, if we fail to write the settings file.
- Use tokio's async runtime and set a hardcoded limit of 4 worker threads, which is already more than enough.
- Add a debug message, when using `pueue wait` or `pueue wait -g some_group`, but there're no tasks in the group.
- Reworked shutdown, restoration and cleanup logic.
- Rename `Index` to `Id` in `pueue status` to free up screen space.
- Remove `Exitcode` column in `pueue status` and include exitcode into `Failed` status to free up screen space.
- You can no longer remove groups, if there are still tasks assigned to that group.

### Datastructures

A whole lot of Pueue's internal datastructures have been refactored.
The main goal of this was to prevent impossible/invalid states wherever possible.

Overall, this resulted in sleaker und much better maintainable code. However, this broke backwards compatibility to pre-v1.0 at numerous places.

- Json structure of the `Task` struct changed significantly, as data depending on the current status has been moved into the `TaskStatus` enum.
- Many messages have been touched, as several new enums have been introduced and many fields have been removed.

### Fixed

- Handle very rare race-condition, where tasks with failed dependencies start anyway.
- `pueue log --json` now works again. [#186](https://github.com/Nukesor/pueue/issues/186)
    By default, only a few lines of output will be provided, but this can be configured via the `--full` and `--lines` option.
- Use crossbeam mpsc channels, which results in faster response time for client connections.

### Removed

- Removed the `enqueue` parameter from callback, as the callback is only fired on finished commands.

## [0.12.2] - 20-04-2021

### Fixed

- Remove task logs on `pueue remove`. [#187](https://github.com/Nukesor/pueue/issues/187)
- Improve Windows support by [oiatz](https://github.com/oiatz). [#114](https://github.com/Nukesor/pueue/issues/114)
- Fix empty output for empty groups when requesting specific group with `status -g $name`. [#190](https://github.com/Nukesor/pueue/issues/190)
- Fix missing output when explicitly requesting default group with `status -g default`. [#190](https://github.com/Nukesor/pueue/issues/190)

## [0.12.1] - 12-03-2021

### Fixed

- Dependant tasks didn't update the id of their dependencies, if a dependency's id was changed via `pueue switch` [#185](https://github.com/Nukesor/pueue/issues/185)

### Changed

- Show the status of the default group, if there are no tasks in the queue.

## [0.12.0] - 10-02-2021

**Info for all packagers:** \
In case you updated your packaging rules for the new layout in v0.11, those changes need to be reverted. \
The new repository layout with workspaces didn't work out that well.
Managing two crates in a single repository in combination with `cargo release` turned out to be quite annoying.

### Added

- `--all-failed` flag for `restart`.
     This will restart all tasks that didn't finish with a `Success` status. [#79](https://github.com/Nukesor/pueue/issues/79)
- New config option `client.dark_mode` by [Mephistophiles](https://github.com/Mephistophiles). [#178](https://github.com/Nukesor/pueue/issues/178)
    Default: `false`. Adds the ability to switch to dark colors instead of regular colors.

### Changed

- Rename/change some flags on the `restart` subcommand.
    1. Rename `--path` to `--edit-path`. The short flag stays the same (`p`).
    2. Rename the short flag for `--start-immediately` to `-k`.
- Dependency bump to pueue-lib `v0.12.1`

### Fixed

- `-s` flag overload on the `restart` command.
    `--start-immediately` and `--stashed` collided.
- Error on BSD due to inability to get username from system registry. [#173](https://github.com/Nukesor/pueue/issues/173)

## [0.11.2] - 01-02-2021

### Changed

- Readability of the `log` command has been further improved.
- Dependency bump to pueue-lib `v0.11.2`

## [0.11.1] - 19-01-2021

### Fixed

- Wrong version (`pueue-v0.11.0-alpha.0`) due to an error in the build process with the new project structure. [#169](https://github.com/Nukesor/pueue/issues/169)

## [0.11.0] - 18-01-2021

### Added

- Add the `--lines` flag to the `log` subcommand.
    This is used to only show the last X lines of each task's stdout and stderr.
- Add the `--full` flag to the `log` subcommand.
    This is used to show the whole logfile of each task's stdout and stderr.
- Add the `--successful-only` flag to the `clean` subcommand.
     This let's keep you all important logs of failed tasks, while freeing up some screen space.

### Changed

- If multiple tasks are selected, `log` now only shows the last few lines for each log.
    You can use the new `--full` option to get the old behavior.

## [0.10.2] - 31-12-2020

### Fixed

- It was possible to remove tasks with active dependants, i.e. tasks which have a dependency and didn't finish yet.
    This didn't lead to any crashes, but could lead to unwanted behavior, since the dependant tasks simply started due to the dependency no longer existing.
    It's however still possible to delete dependencies as long as their dependants are deleted as well.

## [0.10.1] - 29-12-2020

### Fixed

- panic, when using `pueue status` and only having tasks in non-default groups.

## [0.10.0] - 29-12-2020

This release adds a lot of breaking changes!
I tried to clean up, refactor and streamline as much code as possible.

`v0.10.0` aims to be the last release before hitting v1.0.0. \
From that point on I'll try to maintain backward compatibility for as long as possible (v2.0.0).\
Please read this changelog carefully.

### Changed

- Use TLS encryption for all TCP communication. [#52](https://github.com/Nukesor/pueue/issues/52)
- Updated Crossterm and thereby bump the required rust version to `1.48`.
- Extract the shared `secret` into a separate file. [#52](https://github.com/Nukesor/pueue/issues/52)
    This will allow users to publicly sync their config directory between machines.
- Change default secret length from 20 to 512 chars. [#52](https://github.com/Nukesor/pueue/issues/52)
- Lots of internal code cleanup/refactoring/restructuring.
- Exit client with non-zero exit code when getting a failure message from the daemon.
- The `group` list output has been properly styled. 
- Use unix sockets by default on unix systems. [#165](https://github.com/Nukesor/pueue/issues/165)
- Any unix socket code or configuration stuff has been removed, when building for Windows.

### Added

- Add the `shared.host` configuration variable. [#52](https://github.com/Nukesor/pueue/issues/52)
    This finally allows to accept outside connections, but comes with some security implications.
- Create a self-signed ECDSA cert/key for TLS crypto with [rcgen](https://github.com/est31/rcgen). [#52](https://github.com/Nukesor/pueue/issues/52)
- Error messages have been improved in many places.
- `daemon.pause_all_on_failure` config, which actually pauses all groups as soon as a task fails.
- `daemon.pause_group_on_failure` config, which only pauses the group of the affected task instead of everything.
- Users can add some additional information to tasks with the `task add --label $LABEL` option, which will be displayed when calling `pueue status`. [#155](https://github.com/Nukesor/pueue/issues/155)
- `--escape` flag on the `add` subcommand, which takes all given Parameter strings and escapes special characters. [#158](https://github.com/Nukesor/pueue/issues/158)
- Remove `--task-ids` for `wait`. Now it's used the same way as start/kill/pause etc. 
- Add an option `--print-task-id` to only return the task id on `add`. This allows for better scripting. [#151](https://github.com/Nukesor/pueue/issues/151)

### Removed

- Removed the `daemon.pause_on_failure` configuration variable in favor of the other two previously mentioned options.
- Removed the `--port` and `--unix-socket-path` cli flags on client in favor of the `--config` flag.
- Removed the `--port` flag on the daemon in favor of the `--config` flag.

### Fixed

- Properly pass `--config` CLI argument to daemonized `pueued` instance.
- The `--default` flag on the `kill` command has been removed, since this was the default anyway.
    That makes this command's behavior consistent with the `start` and `pause` command.
- Allow the old `kill [task_ids...]` behavior.
    You no longer need the `-t` flag to kill a tasks.
    This broke in one of the previous refactorings.

### Internal

- The default group is now an actual group.

## [0.9.0] - 2020-12-14

### Added

- The `wait` subcommand. This allows you to wait for all tasks in the default queue/ a specific group to finish. [#117](https://github.com/Nukesor/pueue/issues/117)
    On top of this, you can also specify specific tasks ids.
- New client configuration `show_expanded_aliases` (default: `false`).
    Determines whether the original input command or the expanded alias will be shown when calling `status`.
- New `--in-place` option for `restart`, which resets and reuses the existing task instead of creating a new one. [#147](https://github.com/Nukesor/pueue/issues/147)

### Changed

- Don't update the status of tasks with failed dependencies on paused queues.
    This allows to fix dependency chains without having to restart all tasks in combination with the `pause_on_failure` and the new `--in-place` restart option.

### Fixed

- `pause_on_failure` pauses the group of the failed tasks. Previously this always paused the default queue.
- Properly display version when using `-V`. (#143)
- Execute callbacks for tasks with failed dependencies.
- Execute callbacks for tasks that failed to spawn at all.
- Persist state changes when handling tasks that failed to spawn.
- Set proper start/end times for all tasks that failed in any way.

### Changed

- The original user command will be used when editing a task's command.
    As a result of this, aliases will be re-applied after editing a command.

## [0.8.2] - 2020-11-20

### Added

- Add `exit_code` parameter to callback hooks. (#138)
- Add a confirmation message when using `reset` with running tasks by [quebin31](https://github.com/quebin31). [#140](https://github.com/Nukesor/pueue/issues/140)

### Changed

- Update to beta branch of Clap v3. Mainly for better auto-completion scripts.

## [0.8.1] - 2020-10-27

### Added

- Add `start`, `end` and `enqueue` time parameters to callback hooks by [soruh](https://github.com/soruh).
- Config flag to truncate content in 'status'. (#123)

### Fixed

- ZSH completion script fix by [ahkrr](https://github.com/ahkrr).

## [0.8.0] - 2020-10-25

This version adds breaking changes:

- The configuration file structure has been changed. There's now a `shared` section.
- The configuration files have been moved to a dedicated `pueue` subdirectory.

### Added

- Unix socket support [#90](https://github.com/Nukesor/pueue/issues/)
- New option to specify a configuration file on startup for daemon and client.
- Warning messages for removing/killing tasks [#111](https://github.com/Nukesor/pueue/issues/111) by [Julian Kaindl](https://github.com/kaindljulian)
- Better message on `pueue group`, when there are no groups yet.
- Guide on how to connect to remote hosts via ssh port forwarding.

### Changed

- Move a lot of documentation from the README and FAQ into Github's wiki. The docs have been restructured at the same time.
- Never create a default config when starting the client. Only starting the daemon can do that.
- Better error messages when connecting with wrong secret.
- Windows: The configuration file will now also be placed in `%APPDATA%\Local\pueue`.

### Fixed

- Fixed panic, when killing and immediately removing a task. [#119](https://github.com/Nukesor/pueue/issues/119)
- Fixed broken non-responsive daemon, on panic in threads. [#119](https://github.com/Nukesor/pueue/issues/119)
- Don't allow empty commands on `add`.
- The client will never persist/write the configuration file. [#116](https://github.com/Nukesor/pueue/issues/116)
- The daemon will only persist configuration file on startup, if anything changes. [#116](https://github.com/Nukesor/pueue/issues/116)
- (Probably fixed) Malformed configuration file. [#116](https://github.com/Nukesor/pueue/issues/116)

## [0.7.2] - 2020-10-05

### Fixed

- Non-existing tasks were displayed as successfully removed. [#108](https://github.com/Nukesor/pueue/issues/108)
- Remove child process handling logic for MacOs, since the library simply doesn't support this.
- Remove unneeded `config` features and reduce compile time by ~10%. Contribution by [LovecraftianHorror](https://github.com/LovecraftianHorror) [#112](https://github.com/Nukesor/pueue/issues/112)
- Remove futures-timers, effectively reducing compile time by ~14%. [#112](https://github.com/Nukesor/pueue/issues/112)
- Update to comfy-table v1.1.0, reducing compile time by another ~10%. [#112](https://github.com/Nukesor/pueue/issues/112)

### Changed

- Linux process handling now always sends signals to its direct children, if the root process is a `sh -c` process.
  Previously, this behavior was somewhat ambiguous and inconsistent. [#109](https://github.com/Nukesor/pueue/issues/109)

### Added

- Update workflow to build arm binaries.

## [0.7.0] - 2020-07-23

### Added

- New `-e` and `-p` flags to edit tasks on restart.
    `-e` for `command`, `-p` for `path`.
    Both can be added at the same time.

### Changed

- Internal refactoring of the client code. Mostly structure.

### Fixed

- Improved CLI validation.
    Several subcommands accepted empty task id vectors, when they shouldn't.

## [0.6.3] - 2020-07-11

### Changed

- Don't do any code styling, if `stdout` is no tty.

## [0.6.2] - 2020-07-11

### Fixed

- Fix local `stderr` formatting for `log`.
- Fix missing sleep in local `follow` loop, resulting in single core 100% CPU usage.

## [0.6.1] - 2020-06-14

### Changed

- New default behavior for `follow`.
    Implemented by [JP-Ellis](https://github.com/JP-Ellis).
- Delete everything in Pueue's `task_logs` folder on `reset`.

## [0.6.0] - 2020-06-07

### Added

- `pueue_aliases.yml`, which allows some shell-like aliasing.
- `-c` flag for `kill` and `reset`.

## [0.5.1] - 2020-05-31

### Added

- `--children/-c` flag for `start` and `stop`.
  This sends the `SIGSTOP`/`SIGSTART` signal not only to the main process of a task, but also to direct children.
  This is, for instance, useful if you're starting tasks via a shell script.

### Fixed

- Fixed formatting bug in `pueue log`. Fixed by [sourcefrog](https://github.com/sourcefrog).

## [0.5.0] - 2020-05-15

### Added

- Groups! Tasks can now be assigned to a group.
  Each group acts as their own queue and each group has their own setting for parallel task execution.
  Groups can also be paused/resumed individually.
- Added `--group` flag for `status`. This will only print tasks of a specific group
- Add new flags `--default` to `kill`. With this flag only tasks in the default queue will be affected.
- Users can now specify a custom callback that'll be called whenever tasks finish.
- Environment variable capture. Tasks will now start with the variables of the environment `pueue add` is being called in.

### Changed

- `log` now also works on running and paused tasks. It thereby replaces some of `show`'s functionality.
- Rename `show` to `follow`. The `follow` is now only for actually following the output of a single command.
- `follow` (previously `show`) now also reads directly from disk, if `read_local_logs` is set to `true`.
- The `--all` flag now affects all groups AND the default queue for `kill`, `start` and `pause`.

## [0.4.0] - 2020-05-04

### Added

- Dependencies! This adds the `--after [ids]` option. Implemented by [tinou98](https://github.com/tinou98).  
   Task with this option will only be started, if all specified dependencies successfully finish.
  Tasks with failed dependencies will fail as well.
- New state `FailedToStart`. Used if the process cannot be started.
- New state `DependencyFailed`. Used if any dependency of a task fails.
- New config option `read_local_logs`. Default: `true`
  We assume that the daemon and client run on the same machine by default.
  This removes the need to send logs via socket, since the client can directly read the log files.  
   Set to `false` if you, for instance, use Pueue in combination with SSH port forwarding.

### Changed

- Pueue no longer stores log output in its backup files.
- Process log output is no longer permanently stored in memory. This significantly reduced RAM usage for large log outputs. Huge thanks for helping with this to [sourcefrog](https://github.com/sourcefrog)!
- Process log output is compressed in-memory on read from disk. This leads to reduced bandwidth and RAM usage.

## [0.3.1] - 2020-04-10

### Fixed

- Set `start` for processes. (Seems to have broken in 0.2.0)

## [0.3.0] - 2020-04-03

### Added

- `pause_on_failure` configuration flag. Set this to true to pause the daemon as soon as a task fails.
- Add `--stashed` flag to `restart`.
- Add `-p/--path` flag to allow editing of a stashed/queued task's path.
- Better network utilization for `pueue log`.

### Fixed

- Respect `Killed` tasks on `pueue clean`.
- Show `Killed` status in `pueue log`.
- Fix `pueue log` formatting.
- Show daemon status if no tasks exist.
- Better error messages when daemon isn't running.

## [0.2.0] - 2020-03-25

### Added

- New `--delay` flag, which delays enqueueing of a task. Can be used on `start` and `enqueue`. Implemented by [taylor1791](https://github.com/taylor1791).
- `--stashed` flag for `pueue add` to add a task in stashed mode. Implemented by [taylor1791](https://github.com/taylor1791).

### Changed

- Generating completion files moved away from build.rs to the new `pueue completions {shell} {output_dir}` subcommand.
  This seems to be the proper way to generate completion files with clap.
  There is a `build_completions.sh` script to build all completion files to the known location for your convenience.

### Fixed

- Fix `edit` command.
- Several wrong state restorations after restarting pueue.

## [0.1.6] - 2020-02-05

### Fixed

- [BUG] Fix wrong TCP receiving logic.
- Automatically create config directory.
- Fix and reword cli help texts.

## [0.1.5] - 2020-02-02

### Changed

- Basic Windows support. Huge thanks to [Lej77](https://github.com/Lej77) for implementing this!
- Integrate completion script build in `build.rs`.

## [0.1.4] - 2020-01-31

### Changed

- Dependency updates

## [0.1.3] - 2020-01-29

### Changed

- Change table design of `pueue status`.

## [0.1.2] - 2020-01-28

### Fixed

- Handle broken UTF8 in `show` with `-f` and `-e` flags.
- Allow restart of `Killed` processes.

## [0.1.1] - 2020-01-28

### Added

- Add --daemonize flag for daemon to daemonize pueued without using a service manager.
- Add `shutdown` subcommand for client for being able to manually kill the pueue daemon.

### Changed

- Replace prettytables-rs with comfy-table.
- Replace termion with crossterm.
