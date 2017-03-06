from test.helper import (
    execute_add,
    wait_for_process,
)
from pueue.client.displaying import execute_show, execute_log


def test_show(daemon_setup, directory_setup):
    """The show command default executes without failing.

    This implies that the daemon is running and the stdout file in
    `~/.config/pueue/` is properly created.
    """
    execute_add('sleep 120')
    execute_show({'watch': False}, directory_setup[0])


def test_show_empty_queue(daemon_setup, directory_setup):
    """The default `show` command doesn't crash, if there are no entries."""
    execute_show({}, directory_setup[0])


def test_show_specific_non_existing(daemon_setup, directory_setup):
    """The default `show` command doesn't crash, if there are no entries."""
    execute_show({'watch': False, 'keys': [0]}, directory_setup[0])


def test_show_specific(daemon_setup, directory_setup):
    """The show command executes for a specific process without failing."""
    execute_add('sleep 120')
    execute_add('sleep 120')
    execute_show({'watch': False, 'keys': [1]}, directory_setup[0])


def test_show_finished(daemon_setup, directory_setup):
    """The client doesn't crash while trying to show a finished entry."""
    execute_add('ls')
    wait_for_process(0)
    execute_show({'watch': False}, directory_setup[0])


def test_show_specific_finished(daemon_setup, directory_setup):
    """The client doesn't crash while trying to show a specific finished entry."""
    execute_add('ls')
    wait_for_process(0)
    execute_show({'watch': False, 'keys': [0]}, directory_setup[0])


def test_log(daemon_setup, directory_setup):
    """The default `log` command executes without failing.

    This implies that the daemon runs and creates proper log files.
    """
    execute_add('ls')
    wait_for_process(0)
    execute_log({}, directory_setup[0])


def test_log_empty_queue(daemon_setup, directory_setup):
    """The default `log` command doesn't crash, if there are no entries."""
    execute_log({}, directory_setup[0])


def test_log_specific(daemon_setup, directory_setup):
    """The `log` command for specific processes executes without failing."""
    execute_add('ls')
    wait_for_process(0)
    execute_log({'keys': [0]}, directory_setup[0])


def test_log_specific_multiple_succeeded(daemon_setup, directory_setup):
    """The `log` command for multiple specific processes executes without failing."""
    execute_add('ls')
    execute_add('testfailing')
    wait_for_process(1)
    execute_log({'keys': [0, 1]}, directory_setup[0])


def test_log_running(daemon_setup, directory_setup):
    """The `log` command executes without failing.

    This implies that the daemon runs and creates proper log files.
    """
    execute_add('sleep 60')
    execute_log({'keys': [0]}, directory_setup[0])


def test_log_failing(daemon_setup, directory_setup):
    """The `log` command works with keys of running or non existent entries."""
    execute_add('ls')
    wait_for_process(0)
    execute_log({'keys': [0, 1, 3]}, directory_setup[0])


def test_log_multiple_mixed_success(daemon_setup, directory_setup):
    """The `log` command works for failed entries."""
    execute_add('testfailcommand')
    execute_add('ls')
    wait_for_process(1)
    execute_log({'keys': [0, 1, 3]}, directory_setup[0])
