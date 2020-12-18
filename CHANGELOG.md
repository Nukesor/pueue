# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 

### Changed

- Use TLS encryption for TCP communication.
- Extract the shared `secret` into a separate file. This will allow users to publicly sync their config directory between machines.
- Change default secret length from 20 to 500 chars.

### Removed

- `--port` and `--unix-socket-path` cli flags on client. In favor of the `--config` flag.
- `--port` flag on the daemon. IN favor of the `--config` flag.

### Fixed

- Properly pass `--config` CLI argument to daemonized `pueued` instance.

## [0.9.0] - 2020-12-14

### Added

- The `wait` subcommand. This allows you to wait for all tasks in the default queue/ a specific group to finish.
    On top of this, you can also specify specific tasks ids.
- New client configuration `show_expanded_aliases` (default: `false`).
    Determines whether the original input command or the expanded alias will be shown when calling `status`.
- New `--in-place` option for `restart`, which resets and reuses the existing task instead of creating a new one.

### Changed

- Don't update the status of tasks with failed dependencies on paused queues.
    This allows to fix dependency chains without having to restart all tasks in combination with the `pause_on_failure` and the new `--in-place` restart option.

### Fixed

- `pause_on_failure` pauses the group of the failed tasks. Previously this always paused the default queue.
- Properly display version when using `-V`.
- Execute callbacks for tasks with failed dependencies.
- Execute callbacks for tasks that failed to spawn at all.
- Persist state changes when handling tasks that failed to spawn.
- Set proper start/end times for all tasks that failed in any way.

### Changed

- The original user command will be used when editing a task's command.
    As a result of this, aliases will be re-applied after editing a command.

## [0.8.2] - 2020-11-20

### Added

- Add `exit_code` parameter to callback hooks.
- Add a confirmation message when using `reset` with running tasks by [quebin31](https://github.com/quebin31).

### Changed

- Update to beta branch of Clap v3. Mainly for better auto-completion scripts.

## [0.8.1] - 2020-10-27

### Added

- Add `start`, `end` and `enqueue` time parameters to callback hooks by [soruh](https://github.com/soruh).
- Config flag to truncate content in 'status'.

### Fixed

- ZSH completion script fix by [ahkrr](https://github.com/ahkrr).

## [0.8.0] - 2020-10-25

This version adds breaking changes:

- The configuration file structure has been changed. There's now a `shared` section.
- The configuration files have been moved to a dedicated `pueue` subdirectory.

### Added

- Unix socket support (#90)
- New option to specify a configuration file on startup for daemon and client.
- Warning messages for removing/killing tasks (#111) by [Julian Kaindl](https://github.com/kaindljulian)
- Better message on `pueue group`, when there are no groups yet.
- Guide on how to connect to remote hosts via ssh port forwarding.

### Changed

- Move a lot of documentation from the README and FAQ into Github's wiki. The docs have been restructured at the same time.
- Never create a default config when starting the client. Only starting the daemon can do that.
- Better error messages when connecting with wrong secret.
- Windows: The configuration file will now also be placed in `%APPDATA%\Local\pueue`.

### Fixed

- Fixed panic, when killing and immediately removing a task. (#119)
- Fixed broken non-responsive daemon, on panic in threads. (#119)
- Don't allow empty commands on `add`.
- The client will never persist/write the configuration file. (#116)
- The daemon will only persist configuration file on startup, if anything changes. (#116)
- (Probably fixed) Malformed configuration file. (#116)

## [0.7.2] - 2020-10-05

### Fixed

- Non-existing tasks were displayed as successfully removed. (#108)
- Remove child process handling logic for MacOs, since the library simply doesn't support this.
- Remove unneeded `config` features and reduce compile time by ~10%. Contribution by [LovecraftianHorror](https://github.com/LovecraftianHorror) (#112)
- Remove futures-timers, effectively reducing compile time by ~14%. (#112)
- Update to comfy-table v1.1.0, reducing compile time by another ~10%. (#112)

### Changed

- Linux process handling now always sends signals to it's direct children, if the root process is a `sh -c` process.
  Previously, this behavior was somewhat ambiguous and inconsistent. (#109)

### Added

- Update workflow to build arm binaries.

## [0.7.0] - 2020-07-23

### Added

- New `-e` and `-p` flags to edit tasks on restart. `-e` for `command`, `-p` for `path`. Both can be added at the same time.

### Changed

- Internal refactoring of the client code. Mostly structure.

### Fixed

- Improved CLI validation. Several subcommands accepted empty task id vectors, when they shouldn't.

## [0.6.3] - 2020-07-11

### Changed

- Don't do any code styling, if `stdout` is no tty.

## [0.6.2] - 2020-07-11

### Fixed

- Fix local `stderr` formatting for `log`.
- Fix missing sleep in local `follow` loop, resulting in single core 100% CPU usage.

## [0.6.1] - 2020-06-14

### Changed

- New default behavior for `follow`. Implemented by [JP-Ellis](https://github.com/JP-Ellis).
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
