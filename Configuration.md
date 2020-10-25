The default configuration file of Pueue is located in these directories:

- Linux: `$HOME/.config/pueue/pueue.yml`.
- MacOs: `$HOME/Library/Preferences/pueue/pueue.yml`
- Windows: `%APPDATA%\Local\pueue`

A default configuration file will be generated after starting `pueued` for the first time.
You can also force pueue to use a specific configuration file with the `-c` flag for both the daemon and the client.

```yaml
---
shared:
  port: "6924"
  secret: "your_secret"
  pueue_directory: /home/$USER/.local/share/pueue
  use_unix_sockets: false
  unix_sockets_path: /home/$USER/.local/share/pueue/pueue_$USER.socket

client:
  read_local_logs: true
  show_confirmation_questions: false

daemon:
  default_parallel_tasks: 1
  pause_on_failure: false
  callback: ""Task {{ id }}\nCommand: {{ command }}\nPath: {{ path }}\nFinished with status '{{ result }}'\""
  groups:
    cpu: 1
```

### Shared

- `port` The port the daemon listens on and the client connects to in TCP mode.
- `secret` The secret, that's used for authentication
- `pueue_directory` The location Pueue uses for its intermediate files and logs.
- `use_unix_sockets` Whether the daemon should listen on a Unix- or a TCP-socket.
- `unix_socket_path` The path the unix socket is located at.

### Client

- `read_local_logs` If the client runs as the same user (and on the same machine) as the daemon, logs don't have to be sent via the socket but rather read directly.
- `show_confirmation_questions` The client will print warnings that require confirmation for different critical commands.

### Daemon

- `default_parallel_tasks` Determines how many tasks should be processed concurrently.
- `pause_on_failure` If set to `true`, the daemon stops starting new task as soon as a single task fails. Already running tasks will continue.
- `callback` The command that will be called after a task finishes. Can be parameterized
- `groups` This is a list of the groups with their amount of allowed parallel tasks. It's advised to not manipulate this manually, but rather use the `group` subcommand to create and remove groups.

