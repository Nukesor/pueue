use std::path::PathBuf;

use chrono::prelude::*;
use chrono::Duration;
use chrono_english::*;
use clap::Clap;

#[derive(Clap, Debug)]
pub enum SubCommand {
    /// Enqueue a task for execution.
    Add {
        /// The command that should be added.
        #[clap(required = true)]
        command: Vec<String>,

        /// Start the task immediately.
        #[clap(name = "immediate", short, long, conflicts_with = "stashed")]
        start_immediately: bool,

        /// Create the task in stashed state.
        /// Useful to avoid immediate execution if the queue is empty.
        #[clap(name = "stashed", short, long, conflicts_with = "immediate")]
        stashed: bool,

        /// Delays enqueueing the task until <delay> elapses. See "enqueue" for accepted formats.
        #[clap(name = "delay", short, long, conflicts_with = "immediate", parse(try_from_str=parse_delay_until))]
        delay_until: Option<DateTime<Local>>,

        /// Assign the task to a group. Groups kind of act as separate queues.
        /// I.e. all groups run in parallel and you can specify the amount of parallel tasks for each group.
        /// If no group is specified, the default group will be used.
        #[clap(name = "group", short, long)]
        group: Option<String>,

        /// Start the task once all specified tasks have successfully finished.
        /// As soon as one of the dependencies fails, this task will fail as well.
        #[clap(name = "after", short, long)]
        dependencies: Vec<usize>,

        /// Only return the task id instead of a text.
        /// This is useful when scripting and working with dependencies.
        #[clap(short, long)]
        print_task_id: bool,
    },
    /// Remove tasks from the list.
    /// Running or paused tasks need to be killed first.
    Remove {
        /// The task ids to be removed.
        #[clap(required = true)]
        task_ids: Vec<usize>,
    },
    /// Switches the queue position of two commands. Only works on queued and stashed commands.
    Switch {
        /// The first task id.
        task_id_1: usize,
        /// The second task id.
        task_id_2: usize,
    },
    /// Stashed tasks won't be automatically started.
    /// Either enqueue them, to be normally handled or explicitly start them.
    Stash {
        /// The id(s) of the tasks you want to stash.
        #[clap(required = true)]
        task_ids: Vec<usize>,
    },
    /// Enqueue stashed tasks. They'll be handled normally afterwards.
    #[clap(after_help = "DELAY FORMAT:

    The --delay argument must be either a number of seconds or a \"date expression\" similar to GNU \
    `date -d` with some extensions. It does not attempt to parse all natural language, but is \
    incredibly flexible. Here are some supported examples.

    2020-04-01T18:30:00   // RFC 3339 timestamp
    2020-4-1 18:2:30      // Optional leading zeros
    2020-4-1 5:30pm       // Informal am/pm time
    2020-4-1 5pm          // Optional minutes and seconds
    April 1 2020 18:30:00 // English months
    1 Apr 8:30pm          // Implies current year
    4/1                   // American form date
    wednesday 10:30pm     // The closest wednesday in the future at 22:30
    wednesday             // The closest wednesday in the future
    4 months              // 4 months from today at 00:00:00
    1 week                // 1 week at the current time
    1days                 // 1 day from today at the current time
    1d 03:00              // The closest 3:00 after 1 day (24 hours)
    3h                    // 3 hours from now
    3600s                 // 3600 seconds from now
")]
    Enqueue {
        /// The id(s) of the tasks you want to enqueue.
        task_ids: Vec<usize>,

        /// Delay enqueuing the tasks until <delay> elapses. See DELAY FORMAT below.
        #[clap(name = "delay", short, long, parse(try_from_str=parse_delay_until))]
        delay_until: Option<DateTime<Local>>,
    },

    /// Resume operation of specific tasks or groups of tasks.
    /// By default, this resumes the default queue and all its tasks.
    /// Can also be used force-start specific tasks.
    #[clap(verbatim_doc_comment)]
    Start {
        /// Enforce starting these tasks. Paused tasks will be started again.
        /// This does not affect anything other than these tasks.
        task_ids: Vec<usize>,

        /// Start a specific group and all paused tasks in it.
        #[clap(short, long, conflicts_with = "all")]
        group: Option<String>,

        /// Start a everything (Default queue and all groups)!
        /// All groups will be set to `running` and all paused tasks will be resumed.
        #[clap(short, long)]
        all: bool,

        /// Also resume direct child processes of your paused tasks.
        /// By default only the main process will get a SIGSTART.
        #[clap(short, long)]
        children: bool,
    },

    /// Restart task(s).
    /// Identical tasks will be created and by default enqueued.
    /// By default, a new task will be created.
    Restart {
        /// The tasks you want to restart.
        #[clap(required = true)]
        task_ids: Vec<usize>,

        /// Immediately start the task(s).
        #[clap(short, long, name = "immediate", conflicts_with = "stashed")]
        start_immediately: bool,

        /// Create the task in stashed state.
        /// Useful to avoid immediate execution.
        #[clap(short, long)]
        stashed: bool,

        /// Restart the task by reusing the already existing tasks.
        /// This will overwrite any previous logs of the restarted tasks.
        #[clap(short, long)]
        in_place: bool,

        /// Edit the command of the task before restarting
        #[clap(short, long)]
        edit: bool,

        /// Edit the path of the task before restarting
        #[clap(short, long)]
        path: bool,
    },

    /// Pause either running tasks or specific groups of tasks.
    /// By default, pauses the default queue and all its tasks.
    /// A paused queue (group) won't start any new tasks.
    #[clap(verbatim_doc_comment)]
    Pause {
        /// Pause these specific tasks.
        /// Does not affect the default queue, groups or any other tasks.
        task_ids: Vec<usize>,

        /// Pause a specific group.
        #[clap(short, long, conflicts_with = "all")]
        group: Option<String>,

        /// Pause everything (Default queue and all groups)!
        #[clap(short, long)]
        all: bool,

        /// Don not pause already running tasks and let them finish by themselves,
        /// when pausing with `default`, `all` or `group`.
        #[clap(short, long)]
        wait: bool,

        /// Also pause direct child processes of a task's main process.
        /// By default only the main process will get a SIGSTOP.
        /// This is useful when calling bash scripts, which start other processes themselves.
        /// This operation is not recursive!
        #[clap(short, long)]
        children: bool,
    },

    /// Kill specific running tasks or various groups of tasks.
    Kill {
        /// The tasks that should be killed.
        #[clap(short, long)]
        task_ids: Vec<usize>,

        /// Kill all running tasks in the default queue. Pause the default queue.
        #[clap(short, long, conflicts_with = "group", conflicts_with = "all")]
        default: bool,

        /// Kill all running in a group. Pauses the group.
        #[clap(short, long, conflicts_with = "all")]
        group: Option<String>,

        /// Kill ALL running tasks. This also pauses everything
        #[clap(short, long)]
        all: bool,

        /// Send the SIGTERM signal to all children as well.
        /// Useful when working with shell scripts.
        #[clap(short, long)]
        children: bool,
    },

    /// Send something to a task. Useful for sending confirmations such as 'y\n'.
    Send {
        /// The id of the task.
        task_id: usize,

        /// The input that should be sent to the process.
        input: String,
    },

    /// Edit the command or path of a stashed or queued task.
    /// This edits the command of the task by default.
    #[clap(verbatim_doc_comment)]
    Edit {
        /// The id of the task.
        task_id: usize,

        /// Edit the path of the task.
        #[clap(short, long)]
        path: bool,
    },

    /// Manage groups.
    /// By default, this will simply display all known groups.
    Group {
        /// Add a group
        #[clap(short, long, conflicts_with = "remove")]
        add: Option<String>,

        /// Remove a group.
        /// This will move all tasks in this group to the default group!
        #[clap(short, long)]
        remove: Option<String>,
    },

    /// Display the current status of all tasks.
    Status {
        /// Print the current state as json to stdout.
        /// This does not include stdout/stderr of tasks.
        /// Use `log -j` if you want everything.
        #[clap(short, long)]
        json: bool,

        #[clap(short, long)]
        /// Only show tasks of a specific group
        group: Option<String>,
    },

    /// Display the log output of finished tasks.
    /// Prints either all logs or only the logs of specified tasks.
    Log {
        /// Specify for which specific tasks you want to see the output.
        task_ids: Vec<usize>,
        /// Print the current state as json.
        /// Includes EVERYTHING.
        #[clap(short, long)]
        json: bool,
    },

    /// Follow the output of a currently running task.
    /// This command works like tail -f.
    Follow {
        /// The id of the task you want to watch.
        /// If no or multiple tasks are running, you have to specify the id.
        /// If only a single task is running, you can omit the id.
        task_id: Option<usize>,

        /// Show stderr instead of stdout.
        #[clap(short, long)]
        err: bool,
    },

    /// Wait until tasks are finished. This can be quite useful for scripting.
    /// By default, this will wait for all tasks in the default queue to finish.
    /// Note: This will also wait for all tasks that aren't somehow 'Done'.
    /// Includes: [Paused, Stashed, Locked, Queued, ...]
    Wait {
        /// This allows you to wait for specific tasks to finish.
        #[clap(short, long, conflicts_with = "group", conflicts_with = "all")]
        task_ids: Option<Vec<usize>>,

        /// Wait for all tasks in a specific group
        #[clap(short, long, conflicts_with = "all")]
        group: Option<String>,

        /// Wait for all tasks across all groups and the default queue.
        #[clap(short, long)]
        all: bool,

        /// Don't show any log output while waiting
        #[clap(short, long)]
        quiet: bool,
    },

    /// Remove all finished tasks from the list (also clears logs).
    Clean,

    /// Kill all running tasks on user behalf, remove all tasks and reset max_task_id.
    Reset {
        /// Send the SIGTERM signal to all children as well.
        /// Useful when working with shell scripts.
        #[clap(short, long)]
        children: bool,
        /// Force killing all the running tasks without confirmation.
        #[clap(short, long)]
        force: bool,
    },

    /// Remotely shut down the daemon. Should only be used if the daemon isn't started by a service manager.
    Shutdown,

    /// Set the amount of allowed parallel tasks.
    Parallel {
        /// The amount of allowed parallel tasks.
        #[clap(validator=min_one)]
        parallel_tasks: usize,

        /// Specify the amount of parallel tasks for a group.
        #[clap(name = "group", short, long)]
        group: Option<String>,
    },

    /// Generates shell completion files.
    /// This can be ignored during normal operations.
    Completions {
        /// The target shell. Can be `bash`, `fish`, `powershell`, `elvish` and `zsh`.
        #[clap(arg_enum)]
        shell: Shell,
        /// The output directory to which the file should be written.
        output_directory: PathBuf,
    },
}

#[derive(Clap, Debug, PartialEq)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

#[derive(Clap, Debug)]
#[clap(
    name = "Pueue client",
    about = "Interact with the Pueue daemon",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION")
)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Path to a specific pueue config daemon, that should be used.
    /// This ignores all other config files.
    #[clap(short, long)]
    pub config: Option<PathBuf>,

    #[clap(subcommand)]
    pub cmd: SubCommand,
}

fn parse_delay_until(src: &str) -> Result<DateTime<Local>, String> {
    if let Ok(seconds) = src.parse::<i64>() {
        let delay_until = Local::now() + Duration::seconds(seconds);
        return Ok(delay_until);
    }

    if let Ok(date_time) = parse_date_string(src, Local::now(), Dialect::Us) {
        return Ok(date_time);
    }

    Err(String::from(
        "could not parse as seconds or date expression",
    ))
}

/// Validator function. The input string has to be parsable as int and bigger than 0
fn min_one(value: &str) -> Result<(), String> {
    match value.parse::<usize>() {
        Ok(value) => {
            if value < 1 {
                return Err("You must provide a value that's bigger than 0".into());
            }
            Ok(())
        }
        Err(_) => Err("Failed to parse integer".into()),
    }
}
