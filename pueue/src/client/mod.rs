pub mod cli;
/// All subcommands have their dedicated file and functions in here.
mod commands;
pub(crate) mod display_helper;
/// The [`OutputStyle`](style::OutputStyle) helper, responsible for formatting and styling output
/// based on the current settings.
pub mod style;

pub use commands::handle_command;
