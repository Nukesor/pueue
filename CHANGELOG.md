# v0.2.1
**Features:**
- `pause_on_failure` configuration flag. Set this to true to pause the daemon as soon as a task fails.

**Fixes:**
- Respect `Killed` tasks on `pueue clean`.
- Show daemon status if no tasks exist


# v0.2.0
**Features:**
- New `--delay` flag, which delays enqueueing of a task. Can be used on `start` and `enqueue`.
- `--stashed` flag for `pueue add` to add a task in stashed mode.

**For Packager:**
- Generating completion files moved away from build.rs to the new `pueue completions {shell} {output_dir}` subcommand.
This seems to be the proper way to generate completion files with clap.
There is a `build_completions.sh` script to build all completion files to the known location for your convenience.

**Bug fixes:**
- Fix `edit` command
- Several wrong state restorations after restarting pueue

# v0.1.6
- [BUG] Fix wrong TCP receiving logic
- Automatically create config directory
- Fix and reword cli help texts

# v0.1.5
- Basic Windows support
- Integrate completion script build in build.rs

# v0.1.4
- Dependency updates

# v0.1.3
- Change table design of `pueue status`

# v0.1.2
- Handle broken UTF8 in `show` with `-f` and `-e` flags.
- Allow restart of `Killed` processes

# v0.1.1

- Replace prettytables-rs with comfy-table
- Replace termion with crossterm
- Add --daemonize flag for daemon to daemonize pueued without using a service manager
- Add daemon-shutdown subcommand for client for killing a manually daemonized pueued.
