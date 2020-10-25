Grouping tasks can be useful, whenever your tasks utilize different system resources.  
A possible scenario would be to have an `io` group for tasks that copy large files.\
At the same time there's the `cpu` group, which will execute your cpu-heavy (e.g. reencoding) tasks.

The parallelism setting of `io` could then be set to `1` and `cpu` be set to `2`.\
As a result, there'll always be a single task that copies stuff, while two cpu-heavy tasks try to utilize your cpu as good as possible.\
This can prevent task constellations, which might choke your system, while at the same time maximizing resource utilization.

### Add and remove groups

The addition and removal of groups is managed with the `group` subcommand.

- New groups can be added with the `-a` flag. For instance, `pueue group -a cpu` will create the `cpu` group.
- Groups can be removed with the `-r` flag, e.g. `pueue group -r cpu`.
- Show all existing groups by calling `group` without any parameters, i.e. `pueue group`.

### Add tasks to a group

You can specify in which group a task should be run in with the `add -g` flag.\
For example, `pueue add -g cpu -- 'sleep 60'`.\
The `sleep 60` task will then be run in the `cpu` group.

If no group is specified, the task will just be added to the default queue.

### Specify parallel tasks per group

You can set the amount of parallel tasks per group.
Just call the `parallel` subcommand with the `-g` flag.

For instance, `pueue parallel -g cpu 2`.
The `cpu` group will now always run up to two tasks at the same time.

### Pueue status with groups

By default, the `status` command shows all groups with any tasks.\
If you have too many groups, you can specify a group to only see this group's tasks.

### Pause and resume specific groups

Just as `pause` and `start` pause/resume the whole daemon and all running tasks, you can also pause/resume groups and their tasks.\
To pause a specific group add the `-g` flag.

For instance, `pueue pause -g cpu` will pause all tasks in the `cpu` group. This also stops any new tasks from being started in this group.
