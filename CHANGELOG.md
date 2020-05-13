# v0.5.0
**Features:**
- Groups! Tasks can now be assigned to a group.
    Each group acts as their own queue and each group has their own setting for parallel task execution.
    Groups can also be paused/resumed individually.
- Users can now specify a custom callback that'll be called whenever tasks finish.

**Changes:**
- `log` now also works on running and paused tasks. It thereby replaces some of `show`'s functionality.
- Rename `show` to `follow`. The `follow` is now only for actually following the output of a single command.

**Improvements:**
- Environment variable capture. Tasks will now start with the variables of the environment `pueue add` is being called in.
- `follow` (previously `show`) now also reads directly from disk, if `read_local_logs` is set to `true`.

# v0.4.0
**Features:**
- Dependencies! This adds the `--after [ids]` option.
    Task with this option will only be started, if all specified dependencies successfully finish.
    Tasks with failed dependencies will fail as well.
- New state `FailedToStart`. Used if the process cannot be started.
- New state `DependencyFailed`. Used if any dependency of a task fails.
- New config option `read_local_logs`. Default: `true`
    We assume that the daemon and client run on the same machine by default.
    This removes the need to send logs via socket, since the client can directly read the log files.  
    Set to `false` if you, for instance, use Pueue in combination with SSH port forwarding.

**Improvements:**
- Process log output is no longer permanently stored in memory. This significantly reduced RAM usage for large log outputs.
- Process log output is compressed in-memory on read from disk. This leads to reduced bandwidth and RAM usage.

**Changes:**
- Pueue no longer stores log output in its backup files.

# v0.3.1
**Fixes:**
- Set `start` for processes. (Seems to have broken in 0.2.0)

# v0.3.0
**Features:**
- `pause_on_failure` configuration flag. Set this to true to pause the daemon as soon as a task fails.
- Add `--stashed` flag to `restart`.
- Add `-p/--path` flag to allow editing of a stashed/queued task's path.
- Better network utilization for `pueue log`.

**Fixes:**
- Respect `Killed` tasks on `pueue clean`.
- Show `Killed` status in `pueue log`.
- Fix `pueue log` formatting.
- Show daemon status if no tasks exist.
- Better error messages when daemon isn't running.

# v0.2.0
**Features:**
- New `--delay` flag, which delays enqueueing of a task. Can be used on `start` and `enqueue`.
- `--stashed` flag for `pueue add` to add a task in stashed mode.

**For Packager:**
- Generating completion files moved away from build.rs to the new `pueue completions {shell} {output_dir}` subcommand.
This seems to be the proper way to generate completion files with clap.
There is a `build_completions.sh` script to build all completion files to the known location for your convenience.

**Bug fixes:**
- Fix `edit` command.
- Several wrong state restorations after restarting pueue.

# v0.1.6
- [BUG] Fix wrong TCP receiving logic.
- Automatically create config directory.
- Fix and reword cli help texts.

# v0.1.5
- Basic Windows support.
- Integrate completion script build in `build.rs`.

# v0.1.4
- Dependency updates

# v0.1.3
- Change table design of `pueue status`.

# v0.1.2
- Handle broken UTF8 in `show` with `-f` and `-e` flags.
- Allow restart of `Killed` processes.

# v0.1.1

- Replace prettytables-rs with comfy-table.
- Replace termion with crossterm.
- Add --daemonize flag for daemon to daemonize pueued without using a service manager.
- Add daemon-shutdown subcommand for client for killing a manually daemonized pueued.
