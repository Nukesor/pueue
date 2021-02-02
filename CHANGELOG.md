# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

- Moved into a stand-alone repository for better maintainability.

## [0.12.0] -

### Changed

- Moved into a stand-alone repository for better maintainability.
- Change the packet size to 1.4 Kbyte from 1.5 Kbyte
- Remove unused `group_or_default` function.
- Add LOTS of documentation.
- Hide modules that aren't intended for public use.

## [0.11.2] - 01-02-2021

### Changed

- Use `127.0.0.1` instead of `localhost` as default host.
    This prevents any unforseen consequences if somebody deletes the default `localhost` entry from their `/etc/hosts` file.

## [0.11.0] - 18-01-2020

### Fixed

- Don't parse config path, if it's a directory.
- Error with "Couldn't find config at path {:?}" when passing a directory via `--config`.
- Fixed missing newline between tasks in `log` output.
