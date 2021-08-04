mod add;
mod clean;
mod edit;
mod group;
mod kill;
mod parallel_tasks;
mod pause;
mod remove;
mod restart;
/// Tests regarding state restoration from a previous run.
mod restore;
/// Tests for shutting down the daemon.
mod shutdown;
mod start;
mod stashed;
/// Test that the worker pool environment variables are properly injected.
mod worker_environment_variables;
