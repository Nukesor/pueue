

### Groups

Grouping tasks can be useful, whenever your tasks utilize different system resources.  
A possible scenario would be to have an `io` group for tasks that copy large files, while your cpu-heavy (e.g. reencoding) tasks are in a `cpu` group.
The parallelism setting of `io` could then be set to `1` and `cpu` be set to `2`.

As a result, there'll always be a single task that copies stuff, while two tasks try to utilize your cpu as good as possible.\
This removes the problem of scheduling tasks in a way that the system might get slow.
At the same time, you're able to maximize resource utilization.

