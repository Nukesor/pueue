Running `pueued` on a server and wanting to check on the current progress without having to `ssh` onto the machine is a common scenario.
The best solution (for now) is to bind the remote port/socket to a local port/socket.\

**Reminder:**

- You have to set `read_local_logs` config to `false` in the client config. Otherwise `follow` and `log` won't work.
- You have to set the secret of your client configuration to the same of the remote daemon.

**Tips:**

- It's nice to use a separate configuration file for this. The file can be set via the `-c` flag. You should also consider creating an shell alias for this.
- You can create a systemd job, whose job is to open the ssh connection and to reconnect, whenever the connection goes away.

## Port forwarding

For port this looks like this:

```bash
ssh -L 127.0.0.1:6925:127.0.0.1:6925 $REMOTE_USER@yourhost
```

You can now connect from your local pueue to the remote pueue via port 5252. Just write `pueue -p 5252 status`.

## Unix Socket forwarding

Unix-socket to unix-socket is of course also possible:

```bash
ssh -L /tmp/local.socket:/home/$REMOTE_USER/.local/share/pueue/pueue_nuke.sock $REMOTE_USER@yourhost
```

Just connect via `pueue -u /tmp/local_socket status`.

