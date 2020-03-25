use std::path::PathBuf;

use ::chrono::prelude::*;
use ::chrono::Duration;
use ::chrono_english::*;
use ::structopt::clap::Shell;
use ::structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    /// Enqueue a task for execution
    Add {
        /// The command that should be added.
        #[structopt()]
        command: Vec<String>,

        /// Start the task immediately
        #[structopt(name = "immediate", short, long, conflicts_with = "stash")]
        start_immediately: bool,

        /// Create the task stashed.
        /// Useful to avoid immediate execution if the queue is empty.
        #[structopt(name = "stash", short, long, conflicts_with = "immediate")]
        create_stashed: bool,

        /// Delays enqueueing the task until <delay> elapses. See enqueue for accepted formats.
        #[structopt(name = "delay", short, long, conflicts_with = "immediate", parse(try_from_str=parse_delay_until))]
        delay_until: Option<DateTime<Local>>,
    },
    /// Remove tasks from the list.
    /// Running or paused tasks need to be killed first.
    Remove {
        /// The task ids to be removed
        task_ids: Vec<usize>,
    },
    /// Switches the queue position of two commands. Only works on queued and stashed commands.
    Switch {
        /// The first task id
        task_id_1: usize,
        /// The second task id
        task_id_2: usize,
    },
    /// Stashed tasks won't be automatically started.
    /// Either `enqueue` them, to be normally handled or explicitly `start` them.
    Stash {
        /// The id(s) of the tasks you want to stash
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
        /// The id(s) of the tasks you want to enqueue
        task_ids: Vec<usize>,

        /// Delay enqueuing the tasks until <delay> elapses. See DELAY FORMAT below
        #[structopt(name = "delay", short, long, parse(try_from_str=parse_delay_until))]
        delay_until: Option<DateTime<Local>>,
    },

    /// Wake the daemon from its paused state and continue all paused tasks.
    /// Can be used to resume or start specific tasks.
    Start {
        /// Enforce starting these tasks.
        /// This doesn't affect the daemon or any other tasks and works on a paused deamon.
        #[structopt()]
        task_ids: Vec<usize>,
    },
    /// Enqueue tasks again.
    Restart {
        /// The tasks you want to enqueue again.
        #[structopt()]
        task_ids: Vec<usize>,

        /// Start the task(s) immediately
        #[structopt(name = "immediate", short, long)]
        start_immediately: bool,
    },
    /// Pause the daemon and all running tasks.
    /// A paused daemon won't start any new tasks.
    /// Daemon and tasks can be continued with `start`
    /// Can also be used to pause specific tasks.
    Pause {
        /// Pause the daemon, but let any running tasks finish by themselves.
        #[structopt(short, long, group("pause"), conflicts_with("task_ids"))]
        wait: bool,

        /// Pause these tasks.
        /// Doesn't affect the daemon or any other tasks.
        #[structopt(group("pause"))]
        task_ids: Vec<usize>,
    },
    /// Kill either all or only specific running tasks.
    Kill {
        /// Kill all running tasks, this also pauses the daemon.
        #[structopt(short, long, group("kill"), conflicts_with("task_ids"))]
        all: bool,

        /// The tasks that should be killed.
        #[structopt(group("kill"))]
        task_ids: Vec<usize>,
    },

    /// Send something to a task. Useful for sending confirmations ('y\n')
    Send {
        /// The id of the task
        task_id: usize,

        /// The input that should be sent to the process
        input: String,
    },
    /// Edit the command of a stashed or queued task.
    Edit {
        /// The id of the task
        task_id: usize,
    },

    /// Display the current status of all tasks
    Status {
        /// Print the current state as json to stdout
        /// This doesn't include stdout/stderr of tasks.
        /// Use `log -j` if you want everything
        #[structopt(short, long)]
        json: bool,
    },
    /// Display the log output of finished tasks
    Log {
        /// Specify for which specific tasks you want to see the output
        #[structopt()]
        task_ids: Vec<usize>,
        /// Print the current state as json
        /// Includes EVERYTHING
        #[structopt(short, long)]
        json: bool,
    },
    /// Show the output of a currently running task
    /// This command allows following (like `tail -f`)
    Show {
        /// The id of the task
        task_id: usize,
        /// Continuously print stdout (like `tail -f`)
        #[structopt(short, long)]
        follow: bool,
        /// Like -f, but shows stderr instead of stdeout.
        #[structopt(short, long)]
        err: bool,
    },
    /// Remove all finished tasks from the list (also clears logs).
    Clean,
    /// Kill all running tasks, remove all tasks and reset max_id.
    Reset,
    /// Remotely shut down the daemon. Should only be used if the daemon isn't started by a service manager.
    Shutdown,

    /// Set the amount of allowed parallel tasks
    Parallel {
        /// The amount of allowed paralel tasks
        parallel_tasks: usize,
    },
    /// Generates shell completion files.
    /// Ingore for normal operations
    Completions {
        /// The target shell. Can be `bash`, `fish`, `powershell`, `elvish` and `zsh`.
        shell: Shell,
        /// The output directory to which the file should be written
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
    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    //    /// The url for the daemon. Overwrites the address in the config file
    //    #[structopt(short, long)]
    //    pub address: Option<String>,
    /// The port for the daemon. Overwrites the port in the config file
    #[structopt(short, long)]
    pub port: Option<String>,

    #[structopt(subcommand)]
    pub cmd: SubCommand,
}

fn parse_delay_until(src: &str) -> Result<DateTime<Local>, String> {
    let seconds = src.parse::<i64>();
    if seconds.is_ok() {
        let delay_until = Local::now() + Duration::seconds(seconds.unwrap());
        return Ok(delay_until);
    }

    let date_time = parse_date_string(src, Local::now(), Dialect::Us);
    if date_time.is_ok() {
        return Ok(date_time.unwrap());
    }

    Err(String::from(
        "could not parse as seconds or date expression",
    ))
}
