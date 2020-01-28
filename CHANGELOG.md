# v0.1.1
- Handle broken UTF8 in `show` with `-f` and `-e` flags.
- Allow restart of `Killed` processes

# v0.1.1

- Replace prettytables-rs with comfy-table
- Replace termion with crossterm
- Add --daemonize flag for daemon to daemonize pueued without using a service manager
- Add daemon-shutdown subcommand for client for killing a manually daemonized pueued.
