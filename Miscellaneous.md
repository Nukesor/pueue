- [Aliases](#aliases)
- [Callbacks](#callbacks)
- [Shell completion files](#shell-completion-files)
- [Scripting](#scripting)
    * [JSON support](#json-support)
- [Logs](#logs)

## Aliases

To get basic aliasing, simply put a `pueue_aliases.yml` besides your `pueue.yml`.
Its contents should look something like this:

```yaml
ls: 'ls -ahl'
rsync: 'rsync --recursive --partial --perms --progress'
```

When adding a command to pueue, the **first** word will then be checked for the alias.
This means, that for instance `ls ~/ && ls /` will result in `ls -ahl ~/ && ls /`.\
If you want multiple aliases in a single task, it's probably best to either create a task for each command or to write a custom script.

## Callbacks

You can specify a callback that will be called every time a task finishes.
The callback can be parameterized with some variables.

These are the available variables that can be used to create a command:

- `{{ id }}`
- `{{ command }}`
- `{{ path }}`
- `{{ result }}` (Success, Killed, etc.)
- `{{ group }}`

Example callback:

```yaml
callback: "notify-send \"Task {{ id }}\nCommand: {{ command }}\nPath: {{ path }}\nFinished with status '{{ result }}'\""
```

## Shell completion files

Shell completion files can be created on the fly with `pueue completions $shell $directory`.
There's also a `build_completions.sh` script, which creates all completion files in the `utils/completions` directory.

## Scripting

When calling pueue commands in a script, you might need to sleep for a short amount of time for now.
The pueue server processes requests asynchronously, whilst the TaskManager runs it's own update loop with a small sleep.
(The TaskManager handles everything related to starting, stopping and communicating with processes.)

A sleep in scripts will probably become irrelevant, as soon as this bug in rust-lang is fixed: [issue](https://github.com/rust-lang/rust/issues/39364)

### JSON Support

The Pueue client `status` and `log` commands support JSON output with the `-j` flag.
This can also be used to easily incorporate it into window manager bars, such as i3bar.

## Logs

All logs can be found in `${pueue_directory}/logs`.
Logs of previous Pueue sessions will be created whenever you issue a `reset` or `clean`.  
In case the daemon fails or something goes wrong, the daemon will print to `stdout`/`stderr`.
If the daemon crashes or something goes wrong, please set the debug level to `-vvvv` and create an issue with the log!

If you want to dig right into it, you can compile and run it yourself with a debug build.
This would help me a lot!

