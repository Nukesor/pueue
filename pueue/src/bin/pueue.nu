module pueue_completions {
  def get_status [] {
        pueue status "-j" | from json | get tasks
  }
  def get_ids [] {
      get_status | columns
  }
  def get_groups [] {
      pueue status "-j" | from json | get groups | columns
  }
  def sub_pe_comms [] {
      [
          "add" "remove" "switch" "stash" "enqueue" "start" "restart" "pause" "kill" "send" "edit" "group" "status" "format-status" "log" "follow" "wait" "clean" "reset" "shutdown" "parallel" "completions" "help"
      ]
  }
  def color_complete [] {
    ["always" "auto" "nerver"]
  }
  export extern "pueue remove" [
    ...ids: int@get_ids                               # Kill given IDs
    --help(-h)
  ]
  export extern "pueue switch" [
    id_0: int@get_ids
    id_1: int@get_ids
    --help(-h)
  ]
  export extern "pueue stash" [
    ...ids: int@get_ids                               # Stash given IDs
    --help(-h)
  ]
  export extern "pueue enqueue" [
    ...ids: int@get_ids                              
    --delay(-d): string                              # See `pueue enqueue --help`
    --help(-h)
  ]
  export extern "pueue start" [
    ...ids: int@get_ids                               
    --group(-g): string                              # Resume a specific group and all paused tasks in it. The group will be set to running and its paused tasks will be resumed
    --all(-a)                                       # Resume all groups!
    --children(-q)                                  # Deprecated: this switch no longer has any effect
    --help(-h)
  ]
  export extern "pueue restart" [
    ...ids: int@get_ids
    --all-failed(-a)                                # Restart all failed tasks accross all groups.
    --failed-in-group(-g): string@get_groups        # Like `--all-failed`, but only restart tasks failed tasks of a specific group.
    --start-immediately(-k)                         # Immediately start the tasks, no matter how many open slots there are.
    --stashed(-s)                                   # Set the restarted task to a "Stashed" state.
    --in-place(-i)                                  # Restart the task by reusing the already existing tasks.
    --not-in-place                                  # Restart the task by creating a new identical tasks.
    --edit(-e)                                      # Edit the tasks' commands before restarting
    --edit-path(-p)                                 # Edit the tasks' paths before restarting
    --edit-label(-l)                                # Edit the tasks' labels before restarting
    --help(-h) 
  ]
  export extern "pueue pause" [
    ...ids: int@get_ids
    --group(-g): string@get_groups                  # Pause a specific group
    --all(-a)                                       # Pause all groups!
    --wait(-w)                                      # Only pause the specified group and let already running tasks finish by themselves
    --children(-c)                                  # Deprecated: this switch no longer has any effect
    --help(-h)
  ]
  export extern "pueue kill" [
    ...ids: int@get_ids                              
    --group(-g): string@get_groups                  # Kill a specific group
    --all(-a)                                       # Kill  all groups!
    --children(-c)                                  # Deprecated: this switch no longer has any effect
    --signal(-s): string                            # Send a UNIX signal instead of simply killing the process.
    --help(-h)
  ]
  export extern "pueue send" [
    id: int@get_ids
    input: string                                   # The input that should be sent to the process
    --help(-h)
  ]
  export extern "pueue edit" [
    ids: int@get_ids
    --command(-c)                                   # Edit the task's command
    --path(-p)                                      # Edit the task's path
    --label                                         # Edit the task's label
    --help(-h)
  ]
  def pe_group_compls [] {
    ["add" "remove"]
  }
  export extern "pueue group" [
    add?: string@pe_group_compls                    # Add a group by name
    remove?: string@pe_group_compls                 # Remove a group by name. This will move all tasks in this group to the default group!
    --json(-j)                                      # Print the list of groups as json
    --help(-h)
  ]
  export extern "pueue group add" [
    name: string                                    # Name
    --parallel(-p): int                             # Set the amount of parallel tasks this group can have
    --help(-h)
  ]
  export extern "pueue group remove" [
    name: string@get_groups                         # Name
    --help(-h)
  ]
  export extern "pueue status" [
    query?: string                                   # See `pueue status -h`
    --json(-j)                                      # Print the current state as json to stdout.
    --group(-g): string@get_groups                   # Only show tasks of a specific group
    --help(-h)
  ]
  export extern "pueue format-status" [
    --group(-g): string@get_groups                  # Only show tasks of a specific group
    --help(-h)
  ]
  export extern "pueue log" [
    ...ids: int@get_ids
    --json(-j)
    --lines(-l): int                                # Only print the last X lines of each task's output.
    --full(-f)                                      # Show the whole output
    --help(-h)
  ]
  export extern "pueue follow" [
    id: int@get_ids
    --lines(-l): int                                # Only print the last X lines of the output before following
    --help(-h)
  ]
  export extern "pueue wait" [
    ...ids: int@get_ids
    --group(-g): string@get_groups                  # Wait for all tasks in a specific group
    --all(-a)                                       # Wait for all tasks across all groups and the default group
    --quiet(-q)                                     # Don't show any log output while waiting
    --status(-s): string                            # Wait for tasks to reach a specific task status
    --help(-h)
  ]
  export extern "pueue clean" [
    --successful-only(-s)                           # Only clean tasks that finished successfull
    --group(-g): string@get_groups                  # Only clean tasks of a specific group
    --help(-h)
  ]

  export extern "pueue reset" [
    --children(-c)                                  # Deprecated: this switch no longer has any effect
    --force(-f)                                     # Don't ask for any confirmation
    --help(-h)
  ]
  export extern "pueue shutdown" [
    --help(-h)
  ]
  export extern "pueue parallel" [
    amount: int                                     # The amount of allowed parallel tasks
    --group(-g): string@get_groups                  # Set the amount for a specific group
    --help(-h)
  ]
  def ONLY_NUSHELL [] {
    [nushell]
  }
  export extern "pueue completions" [
      shell: string@ONLY_NUSHELL                      # The target shell [possible values: bash, elvish, fish, power-shell, zsh, nushell]
    out_dir: path                                   # The output directory to which the file should be written
  ]
  export extern "pueue" [
    add?: string@sub_pe_comms                        # Enqueue a task for execution.
    remove?: string@sub_pe_comms                    # Remove tasks from the list. Running or paused tasks need to be killed first
    switch?: string@sub_pe_comms                    # Switches the queue position of two commands.
    stash?: string@sub_pe_comms                     # Stashed tasks won't be automatically started.
    enqueue?: string@sub_pe_comms                   # Enqueue stashed tasks. They'll be handled normally afterwards
    start?: string@sub_pe_comms                     # Resume operation of specific tasks or groups of tasks.
    restart?: string@sub_pe_comms                   # Restart failed or successful task(s).
    pause?: string@sub_pe_comms                     # Either pause running tasks or specific groups of tasks.
    kill?: string@sub_pe_comms                      # Kill specific running tasks or whole task groups.
    send?: string@sub_pe_comms                      # Send something to a task. Useful for sending confirmations such as 'y\n'
    edit?: string@sub_pe_comms                      # Edit the command, path or label of a stashed or queued task.
    group?: string@sub_pe_comms                     # Use this to add or remove groups.By default, this will simply display all known groups.
    status?: string@sub_pe_comms                    # Display the current status of all tasks
    format_status?: string@sub_pe_comms             # Accept a list or map of JSON pueue tasks via stdin and display it just like "pueue status".
    log?: string@sub_pe_comms                       # Display the log output of finished tasks.
    follow?: string@sub_pe_comms                    # Follow the output of a currently running task.
    wait?: string@sub_pe_comms                      # Wait until tasks are finished. By default, this will wait for all tasks in the default group to finish.
    clean?: string@sub_pe_comms                     # Remove all finished tasks from the list
    reset?: string@sub_pe_comms                     # Kill all tasks, clean up afterwards and reset EVERYTHING!
    shutdown?: string@sub_pe_comms                  # Remotely shut down the daemon.
    parallel?: string@sub_pe_comms                  # Set the amount of allowed parallel tasks
    completions?: string@sub_pe_comms               # Generates shell completion files.
    help?: string@sub_pe_comms                      # Print this message or the help of the given subcommand(s)
    --verbose(-v)                                   # Verbose mode (-v, -vv, -vvv)
    --color                                         # Colorize the output
    --config(-c)                                    # Path to a specific pueue config file to use.
    --profile(-p)                                   # The name of the profile that should be loaded from your config file
    --help(-h)                                      # Print help
    --version(-V)                                   # Print version
  ]
}
use pueue_completions *
