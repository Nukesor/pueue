---
name: Bug report
about: Create a report to help us improve
title: "[BUG]"
labels: bug
assignees: Nukesor

---

First of all, please read the [getting started guide](https://github.com/Nukesor/pueue/wiki/Get-started).
Especially, if you have a problem with [character escaping](https://github.com/Nukesor/pueue/wiki/Get-started#shell-escaping) or [failing shell commands](https://github.com/Nukesor/pueue/wiki/Common-Pitfalls-and-Debugging#first-step)!

Otherwise feel free to go ahead :)

#### Describe the bug

A clear and concise description of what the bug is.

#### Steps to reproduce the bug

1. I added a task with `pueue add -- this is the command`
2. Then I did ...

#### Expected behavior

A clear and concise description of what you expected to happen.

#### Logs/Output

If applicable, add some program output to help explain your problem.

If there's a panic or undefined behavior, consider adding your `pueued` logs.
You can set the daemon to verbose via `pueue -vvv`.

Don't forget to remove potentially sensitive output though.

#### Additional context

- Operating System
- Pueue version. Can be seen at the top of `pueue -h` output
