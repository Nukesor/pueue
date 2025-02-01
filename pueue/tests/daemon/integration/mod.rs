mod add;
mod aliases;
mod callback;
mod clean;
mod dependencies;
mod edit;
mod environment_variables;
mod group;
mod kill;
mod log;
mod parallel_tasks;
mod pause;
mod priority;
mod remove;
mod reset;
mod restart;
/// Tests regarding state restoration from a previous run.
mod restore;
/// Tests for shutting down the daemon.
mod shutdown;
mod socket_permissions;
mod spawn;
mod start;
mod stashed;
/// Test that the worker pool environment variables are properly injected.
mod worker_environment_variables;
