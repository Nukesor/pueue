// This lint is generating way too many false-positives.
// Ignore it for now.
#![allow(clippy::assigning_clones)]
#![doc = include_str!("../README.md")]
pub mod client;
pub mod daemon;
