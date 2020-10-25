# Frequently Asked Questions

## The Project Does Not Compile
Do you have Rust version `>=1.39` installed?
If that's not the case, there should also be warning at the very top of your `cargo build` output.

If you have Rust version `>=1.39`, please file a bug report.

## What's The Advantage Over Using A Terminal Multiplexer

- The ability to queue commands and not start them all at once
- You can specify how many parallel tasks you want
- Easy pausing/resuming of tasks
- A very pretty printed table with a good overview
- You don't need to attach to a multiple tmux sessions, just check your tasks' status via pueue.

Additionally the long term plan is:

- to allow remote connection to servers. No need to ssh onto a server, you can manipulate and check the status directly on your local machine.

A lot of its functionality is convenience. Using your shell's tools is possible, but IMO, having something that's specifically designed for this task is more efficient and definitely more convenient.

For example, one of my very regular use cases is Movie encoding. In this case I want:

- at most two parallel encodes (otherwise performance degrades).
- them to be processed one after another
- to see at first glance, whether a command fails
- an easy way to look at output
- everything to be in a uniform interface
- it to look pretty and clear
- being able to pause/resume everything in case I need to run something with full power on my server right NOW
- I probably forgot a few things

I used tmux for this stuff all the time before writing Pueue.
However, after using it for a really long time, it just kept feeling annoying and inconvenient.
Up to the point I couldn't bare it any longer and decided to write something that's better suited for such scenarios.

Additionally, I already used Pueue as a TaskManager in combination with some other tools (It has a simple JSON API via TCP).
Those tools need to be adjusted for the Rust rewrite, but back then it worked like a charm.
