pub mod cli;
#[allow(clippy::module_inception)]
pub mod client;
/// All subcommands have their dedicated file and functions in here.
mod commands;
pub(crate) mod display_helper;
/// The [`OutputStyle`] helper, responsible for formatting and styling output based on the current
/// settings.
mod style;

pub use commands::handle_command;
