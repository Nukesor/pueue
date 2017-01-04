from test.helper import (
    execute_add,
    wait_for_process,
)
from pueue.client.displaying import execute_show, execute_log


def test_show(daemon_setup):
    """The show command executes without failing.

    This implies that the daemon is running and the stdout file in /tmp
    is properly created.
    """
    execute_add('sleep 0.5')
    wait_for_process(0)
    execute_show({'watch': False})


def test_log(daemon_setup):
    """The logging command executes without failing.

    This implies that the daemon runs and creates proper log files.
    """
    execute_add('sleep 0.5')
    wait_for_process(0)
    execute_log({})
