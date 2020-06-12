use std::path::PathBuf;

use ::chrono::prelude::*;
use ::chrono::Duration;
use ::chrono_english::*;
use ::structopt::clap::Shell;
use ::structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    /// Enqueue a task for execution.
    Add {
        /// The command that should be added.
        command: Vec<String>,

        /// Start the task immediately.
        #[structopt(name = "immediate", short, long, conflicts_with = "stash")]
        start_immediately: bool,

        /// Create the task in stashed state.
        /// Useful to avoid immediate execution if the queue is empty.
        #[structopt(name = "stashed", short, long, conflicts_with = "immediate")]
        stashed: bool,

        /// Delays enqueueing the task until <delay> elapses. See enqueue for accepted formats.
        #[structopt(name = "delay", short, long, conflicts_with = "immediate", parse(try_from_str=parse_delay_until))]
        delay_until: Option<DateTime<Local>>,

        /// Assign the task to a group. Groups kind of act as separate queues.
        /// I.e. all groups run in parallel and you can specify the amount of parallel tasks for each group.
        /// If no group is specified, the default group will be used.
        #[structopt(name = "group", short, long)]
        group: Option<String>,

        /// Start the task once all specified tasks have successfully finished.
        /// As soon as one of the dependencies fails, this task will fail as well.
        #[structopt(name = "after", short, long)]
        dependencies: Vec<usize>,
    },
    /// Remove tasks from the list.
    /// Running or paused tasks need to be killed first.
    Remove {
        /// The task ids to be removed.
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
    /// Either `enqueue` them, to be normally handled or explicitly `start` them.
    Stash {
        /// The id(s) of the tasks you want to stash.
        task_ids: Vec<usize>,
    },
    /// Enqueue stashed tasks. They'll be handled normally afterwards.
    #[structopt(after_help = "DELAY FORMAT:

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
        #[structopt(name = "delay", short, long, parse(try_from_str=parse_delay_until))]
        delay_until: Option<DateTime<Local>>,
    },

    /// Resume operation of specific tasks or groups of tasks.
    /// Without any parameters, resumes the default queue and all it's tasks.
    /// Can also be used force specific tasks to start.
    #[structopt(verbatim_doc_comment)]
    Start {
        /// Enforce starting these tasks. Paused tasks will be started again.
        /// This doesn't affect anything other than these tasks.
        task_ids: Vec<usize>,

        /// Start a specific group and all paused tasks in it.
        #[structopt(short, long, group("start"))]
        group: Option<String>,

        /// Start a everything (Default queue and all groups)!
        /// All groups will be set to `running` and all paused tasks will be resumed.
        #[structopt(short, long, group("start"))]
        all: bool,

        /// Also resume direct child processes of your paused tasks.
        /// By default only the main process will get a SIGSTART.
        #[structopt(short, long, group("start"))]
        children: bool,
    },

    /// Restart task(s).
    /// Identical tasks will be created and instantly queued (unless specified otherwise).
    Restart {
        /// The tasks you want to restart.
        task_ids: Vec<usize>,

        /// Immediately start the task(s).
        #[structopt(name = "immediate", short, long)]
        start_immediately: bool,

        /// Create the task in stashed state.
        /// Useful to avoid immediate execution.
        #[structopt(name = "stashed", short, long, conflicts_with = "immediate")]
        stashed: bool,
    },

    /// Pause either running tasks or specific groups of tasks.
    /// Without any parameters, pauses the default queue and all it's tasks.
    /// A paused queue (group) won't start any new tasks.
    /// Everything can be resumed with `start`.
    #[structopt(verbatim_doc_comment)]
    Pause {
        /// Pause these specific tasks.
        /// Doesn't affect the default queue, groups or any other tasks.
        task_ids: Vec<usize>,

        /// Pause a specific group.
        #[structopt(short, long, group("pause"))]
        group: Option<String>,

        /// Pause everything (Default queue and all groups)!
        #[structopt(short, long, group("pause"))]
        all: bool,

        /// Don't pause already running tasks and let them finish by themselves,
        ///  when pausing with `default`, `all` or `group`.
        #[structopt(short, long, group("pause"))]
        wait: bool,

        /// Also pause direct child processes of a task's main process.
        /// By default only the main process will get a SIGSTOP.
        /// This is useful when calling bash scripts, which start other processes themselves.
        /// This operation is not recursive!
        #[structopt(short, long, group("pause"))]
        children: bool,
    },

    /// Kill specific running tasks or various groups of tasks.
    Kill {
        /// The tasks that should be killed.
        task_ids: Vec<usize>,

        /// Kill all running tasks in the default queue. Pause the default queue.
        #[structopt(short, long, group("kill"))]
        default: bool,

        /// Kill all running in a group. Pauses the group.
        #[structopt(short, long, group("kill"))]
        group: Option<String>,

        /// Kill ALL running tasks. This also pauses everything
        #[structopt(short, long, group("kill"))]
        all: bool,

        /// Send the SIGTERM signal to all children as well.
        /// Useful when working with shell scripts.
        #[structopt(short, long, group("kill"))]
        children: bool,
    },

    /// Send something to a task. Useful for sending confirmations ('y\n').
    Send {
        /// The id of the task.
        task_id: usize,

        /// The input that should be sent to the process.
        input: String,
    },

    /// Edit the command or path of a stashed or queued task.
    /// This edits the command of the task by default.
    #[structopt(verbatim_doc_comment)]
    Edit {
        /// The id of the task.
        task_id: usize,

        /// Edit the path of the task.
        #[structopt(short, long)]
        path: bool,
    },

    /// Manage groups.
    /// Without any flags, this will simply display all known groups.
    Group {
        /// Add a group
        #[structopt(short, long)]
        add: Option<String>,

        /// Remove a group.
        /// This will move all tasks in this group to the default group!
        #[structopt(short, long)]
        remove: Option<String>,
    },

    /// Display the current status of all tasks.
    Status {
        /// Print the current state as json to stdout.
        /// This doesn't include stdout/stderr of tasks.
        /// Use `log -j` if you want everything.
        #[structopt(short, long)]
        json: bool,

        #[structopt(short, long)]
        /// Only show tasks of a specific group
        group: Option<String>,
    },

    /// Display the log output of finished tasks.
    Log {
        /// Specify for which specific tasks you want to see the output.
        task_ids: Vec<usize>,
        /// Print the current state as json.
        /// Includes EVERYTHING.
        #[structopt(short, long)]
        json: bool,
    },

    /// Follow the output of a currently running task.
    /// This command works like `tail -f`.
    Follow {
        /// The id of the task.  If None, it will default to the running task if
        /// only one is running.  If no or multiple tasks are running, a message
        /// will be displayed.
        task_id: Option<usize>,

        /// Show stderr instead of stdout.
        #[structopt(short, long)]
        err: bool,
    },

    /// Remove all finished tasks from the list (also clears logs).
    Clean,

    /// Kill all running tasks, remove all tasks and reset max_task_id.
    Reset {
        /// Send the SIGTERM signal to all children as well.
        /// Useful when working with shell scripts.
        #[structopt(short, long)]
        children: bool,
    },

    /// Remotely shut down the daemon. Should only be used if the daemon isn't started by a service manager.
    Shutdown,

    /// Set the amount of allowed parallel tasks.
    Parallel {
        /// The amount of allowed parallel tasks.
        #[structopt(validator=min_one)]
        parallel_tasks: usize,

        /// Specify the amount of parallel tasks for a group.
        #[structopt(name = "group", short, long)]
        group: Option<String>,
    },

    /// Generates shell completion files.
    /// This can be ignored during normal operations.
    Completions {
        /// The target shell. Can be `bash`, `fish`, `powershell`, `elvish` and `zsh`.
        shell: Shell,
        /// The output directory to which the file should be written.
        output_directory: PathBuf,
    },
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "Pueue client",
    about = "Interact with the Pueue daemon",
    author = "Arne Beer <contact@arne.beer>"
)]
pub struct Opt {
    // The number of occurrences of the `v/verbose` flag.
    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    //    /// The url for the daemon. Overwrites the address in the config file
    //    #[structopt(short, long)]
    //    pub address: Option<String>,
    /// The port for the daemon. Overwrites the port in the config file.
    #[structopt(short, long)]
    pub port: Option<String>,

    #[structopt(subcommand)]
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
fn min_one(value: String) -> Result<(), String> {
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
