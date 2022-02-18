use std::path::PathBuf;

use chrono::prelude::*;
use chrono::Duration;
use chrono_english::*;
use clap::{ArgEnum, Parser, ValueHint};

use pueue_lib::network::message::Signal;

#[derive(Parser, Debug)]
pub enum SubCommand {
    /// Enqueue a task for execution.
    Add {
        /// The command to be added.
        #[clap(required = true, value_hint = ValueHint::CommandWithArguments)]
        command: Vec<String>,

        /// Specify current working directory.
        #[clap(name = "working-directory", short = 'w', long, value_hint = ValueHint::DirPath)]
        working_directory: Option<PathBuf>,

        /// Escape any special shell characters (" ", "&", "!", etc.).
        /// Beware: This implicitly disables nearly all shell specific syntax ("&&", "&>").
        #[clap(short, long)]
        escape: bool,

        /// Immediately start the task.
        #[clap(name = "immediate", short, long, conflicts_with = "stashed")]
        start_immediately: bool,

        /// Create the task in Stashed state.
        /// Useful to avoid immediate execution if the queue is empty.
        #[clap(name = "stashed", short, long, conflicts_with = "immediate")]
        stashed: bool,

        /// Prevents the task from being enqueued until <delay> elapses. See "enqueue" for accepted formats.
        #[clap(name = "delay", short, long, conflicts_with = "immediate", parse(try_from_str=parse_delay_until))]
        delay_until: Option<DateTime<Local>>,

        /// Assign the task to a group. Groups kind of act as separate queues.
        /// I.e. all groups run in parallel and you can specify the amount of parallel tasks for each group.
        /// If no group is specified, the default group will be used.
        #[clap(name = "group", short, long)]
        group: Option<String>,

        /// Start the task once all specified tasks have successfully finished.
        /// As soon as one of the dependencies fails, this task will fail as well.
        #[clap(name = "after", short, long, multiple_values(true))]
        dependencies: Vec<usize>,

        /// Add some information for yourself.
        /// This string will be shown in the "status" table.
        /// There's no additional logic connected to it.
        #[clap(short, long)]
        label: Option<String>,

        /// Only return the task id instead of a text.
        /// This is useful when scripting and working with dependencies.
        #[clap(short, long)]
        print_task_id: bool,
    },
    /// Remove tasks from the list.
    /// Running or paused tasks need to be killed first.
    #[clap(alias("rm"))]
    Remove {
        /// The task ids to be removed.
        #[clap(required = true)]
        task_ids: Vec<usize>,
    },
    /// Switches the queue position of two commands.
    /// Only works on queued and stashed commands.
    Switch {
        /// The first task id.
        task_id_1: usize,
        /// The second task id.
        task_id_2: usize,
    },
    /// Stashed tasks won't be automatically started.
    /// You have to enqueue them or start them by hand.
    Stash {
        /// Stash these specific tasks.
        #[clap(required = true)]
        task_ids: Vec<usize>,
    },
    /// Enqueue stashed tasks. They'll be handled normally afterwards.
    #[clap(after_help = "DELAY FORMAT:

    The --delay argument must be either a number of seconds or a \"date expression\" similar to GNU \
    \"date -d\" with some extensions. It does not attempt to parse all natural language, but is \
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
        /// Enqueue these specific tasks.
        task_ids: Vec<usize>,

        /// Delay enqueuing these tasks until <delay> elapses. See DELAY FORMAT below.
        #[clap(name = "delay", short, long, parse(try_from_str=parse_delay_until))]
        delay_until: Option<DateTime<Local>>,
    },

    /// Resume operation of specific tasks or groups of tasks.
    /// By default, this resumes the default group and all its tasks.
    /// Can also be used force-start specific tasks.
    #[clap(verbatim_doc_comment)]
    Start {
        /// Start these specific tasks. Paused tasks will resumed.
        /// Queued or Stashed tasks will be force-started.
        task_ids: Vec<usize>,

        /// Resume a specific group and all paused tasks in it.
        /// The group will be set to running and its paused tasks will be resumed.
        #[clap(short, long, conflicts_with = "all")]
        group: Option<String>,

        /// Resume all groups!
        /// All groups will be set to running and paused tasks will be resumed.
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
    #[clap(alias("re"))]
    Restart {
        /// Restart these specific tasks.
        task_ids: Vec<usize>,

        /// Restart all failed tasks accross all groups.
        /// Nice to use in combination with `-i/--in-place`.
        #[clap(short, long)]
        all_failed: bool,

        /// Like `--all-failed`, but only restart tasks failed tasks of a specific group.
        /// The group will be set to running and its paused tasks will be resumed.
        #[clap(short = 'g', long, conflicts_with = "all-failed")]
        failed_in_group: Option<String>,

        /// Immediately start the tasks, no matter how many open slots there are.
        /// This will ignore any dependencies tasks may have.
        #[clap(short = 'k', long, conflicts_with = "stashed")]
        start_immediately: bool,

        /// Set the restarted task to a "Stashed" state.
        /// Useful to avoid immediate execution.
        #[clap(short, long)]
        stashed: bool,

        /// Restart the task by reusing the already existing tasks.
        /// This will overwrite any previous logs of the restarted tasks.
        #[clap(short, long)]
        in_place: bool,

        /// Restart the task by creating a new identical tasks.
        /// Only applies, if you have the restart_in_place configuration set to true.
        #[clap(long)]
        not_in_place: bool,

        /// Edit the tasks' command before restarting.
        #[clap(short, long)]
        edit: bool,

        /// Edit the tasks' path before restarting.
        #[clap(short = 'p', long)]
        edit_path: bool,
    },

    /// Either pause running tasks or specific groups of tasks.
    /// By default, pauses the default group and all its tasks.
    /// A paused queue (group) won't start any new tasks.
    #[clap(verbatim_doc_comment)]
    Pause {
        /// Pause these specific tasks.
        /// Does not affect the default group, groups or any other tasks.
        task_ids: Vec<usize>,

        /// Pause a specific group.
        #[clap(short, long, conflicts_with = "all")]
        group: Option<String>,

        /// Pause all groups!
        #[clap(short, long)]
        all: bool,

        /// Only pause the specified group and let already running tasks finish by themselves.
        #[clap(short, long)]
        wait: bool,

        /// Also pause direct child processes of a task's main process.
        /// By default only the main process will get a SIGSTOP.
        /// This is useful when calling bash scripts, which start other processes themselves.
        /// This operation is not recursive!
        #[clap(short, long)]
        children: bool,
    },

    /// Kill specific running tasks or whole task groups.
    /// Kills all tasks of the default group when no ids are provided.
    Kill {
        /// Kill these specific tasks.
        task_ids: Vec<usize>,

        /// Kill all running tasks in a group. This also pauses the group.
        #[clap(short, long, conflicts_with = "all")]
        group: Option<String>,

        /// Kill all running tasks across ALL groups. This also pauses all groups.
        #[clap(short, long)]
        all: bool,

        /// Send the SIGTERM signal to all children as well.
        /// Useful when working with shell scripts.
        #[clap(short, long)]
        children: bool,

        /// Send a UNIX signal instead of simply killing the process.
        /// DISCLAIMER: This bypasses Pueue's process handling logic!
        ///     You might enter weird invalid states, use at your own descretion.
        #[clap(short, long, ignore_case(true))]
        signal: Option<Signal>,
    },

    /// Send something to a task. Useful for sending confirmations such as 'y\n'.
    Send {
        /// The id of the task.
        task_id: usize,

        /// The input that should be sent to the process.
        input: String,
    },

    /// Edit the command or path of a stashed or queued task.
    /// The command is edited by default.
    #[clap(verbatim_doc_comment)]
    Edit {
        /// The id of the task.
        task_id: usize,

        /// Edit the path of the task.
        #[clap(short, long)]
        path: bool,
    },

    /// Use this to add or remove groups.
    /// By default, this will simply display all known groups.
    Group {
        #[clap(subcommand)]
        cmd: Option<GroupCommand>,
    },

    /// Display the current status of all tasks.
    Status {
        /// Print the current state as json to stdout.
        /// This does not include the output of tasks.
        /// Use `log -j` if you want everything.
        #[clap(short, long)]
        json: bool,

        #[clap(short, long)]
        /// Only show tasks of a specific group
        group: Option<String>,
    },

    /// Accept a list or map of JSON pueue tasks via stdin and display it just like "status".
    /// A simple example might look like this:
    /// pueue status --json | jq -c '.tasks' | pueue format-status
    #[clap(after_help = "DISCLAIMER:
    This command is a temporary workaround until a proper filtering language for \"status\" has
    been implemented. It might be removed in the future.")]
    FormatStatus {
        #[clap(short, long)]
        /// Only show tasks of a specific group
        group: Option<String>,
    },

    /// Display the log output of finished tasks.
    /// When looking at multiple logs, only the last few lines will be shown.
    /// If you want to "follow" the output of a task, please use the "follow" subcommand.
    Log {
        /// View the task output of these specific tasks.
        task_ids: Vec<usize>,

        /// Print the resulting tasks and output as json.
        /// By default only the last lines will be returned unless --full is provided.
        /// Take care, as the json cannot be streamed!
        /// If your logs are really huge, using --full can use all of your machine's RAM.
        #[clap(short, long)]
        json: bool,

        /// Only print the last X lines of each task's output.
        /// This is done by default if you're looking at multiple tasks.
        #[clap(short, long, conflicts_with = "full")]
        lines: Option<usize>,

        /// Show the whole output.
        /// This is the default if only a single task is being looked at.
        #[clap(short, long)]
        full: bool,
    },

    /// Follow the output of a currently running task.
    /// This command works like tail -f.
    #[clap(alias("fo"))]
    Follow {
        /// The id of the task you want to watch.
        /// If no or multiple tasks are running, you have to specify the id.
        /// If only a single task is running, you can omit the id.
        task_id: Option<usize>,

        /// Only print the last X lines of the output before following
        #[clap(short, long)]
        lines: Option<usize>,
    },

    /// Wait until tasks are finished. This can be quite useful for scripting.
    /// By default, this will wait for all tasks in the default group to finish.
    /// Note: This will also wait for all tasks that aren't somehow 'Done'.
    /// Includes: [Paused, Stashed, Locked, Queued, ...]
    Wait {
        /// This allows you to wait for specific tasks to finish.
        task_ids: Vec<usize>,

        /// Wait for all tasks in a specific group
        #[clap(short, long, conflicts_with = "all")]
        group: Option<String>,

        /// Wait for all tasks across all groups and the default group.
        #[clap(short, long)]
        all: bool,

        /// Don't show any log output while waiting
        #[clap(short, long)]
        quiet: bool,
    },

    /// Remove all finished tasks from the list.
    Clean {
        /// Only clean tasks that finished successfully.
        #[clap(short, long)]
        successful_only: bool,

        /// Only clean tasks of a specific group
        #[clap(short, long)]
        group: Option<String>,
    },

    /// Kill all tasks, clean up afterwards and reset EVERYTHING!
    Reset {
        /// Send the SIGTERM signal to all children as well.
        /// Useful when working with shell scripts.
        #[clap(short, long)]
        children: bool,

        /// Don't ask for any confirmation.
        #[clap(short, long)]
        force: bool,
    },

    /// Remotely shut down the daemon. Should only be used if the daemon isn't started by a service manager.
    Shutdown,

    /// Set the amount of allowed parallel tasks.
    /// By default, adjusts the amount of the default group.
    Parallel {
        /// The amount of allowed parallel tasks.
        #[clap(validator=min_one)]
        parallel_tasks: Option<usize>,

        /// Set the amount for a specific group.
        #[clap(name = "group", short, long)]
        group: Option<String>,
    },

    /// Generates shell completion files.
    /// This can be ignored during normal operations.
    Completions {
        /// The target shell.
        #[clap(arg_enum)]
        shell: Shell,
        /// The output directory to which the file should be written.
        #[clap(value_hint = ValueHint::DirPath)]
        output_directory: PathBuf,
    },
}

#[derive(Parser, Debug)]
pub enum GroupCommand {
    /// Add a group by name.
    Add {
        name: String,

        /// Set the amount of parallel tasks this group can have.
        #[clap(short, long, validator = min_one)]
        parallel: Option<usize>,
    },

    /// Remove a group by name.
    /// This will move all tasks in this group to the default group!
    Remove { name: String },
}

#[derive(Parser, ArgEnum, Debug, Clone, PartialEq)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

#[derive(Parser, Debug)]
#[clap(
    name = "Pueue client",
    about = "Interact with the Pueue daemon",
    author,
    version
)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Path to a specific pueue config file to use.
    /// This ignores all other config files.
    #[clap(short, long, value_hint = ValueHint::FilePath)]
    pub config: Option<PathBuf>,

    /// The name of the profile that should be loaded from your config file.
    #[clap(short, long)]
    pub profile: Option<String>,

    #[clap(subcommand)]
    pub cmd: Option<SubCommand>,
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
