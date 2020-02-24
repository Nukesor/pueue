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
        create_stashed: bool
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
    Enqueue {
        /// The id(s) of the tasks you want to enqueue
        task_ids: Vec<usize>,
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
