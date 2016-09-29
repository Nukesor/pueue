from test.helper import (
    command_factory,
    execute_add,
    get_status,
)


def test_start(daemon_setup):
    """Start daemon after it has been paused."""
    command_factory('pause')
    command_factory('start')
    status = get_status()
    assert status['status'] == 'running'


def test_reset_paused(daemon_setup):
    """Reset the daemon."""
    command_factory('pause')
    execute_add({'command': 'sleep 60'})
    execute_add({'command': 'sleep 60'})
    command_factory('reset')
    status = get_status()
    assert status['status'] == 'paused'
    assert status['data'] == 'Queue is empty'


def test_reset_running(daemon_setup):
    """Reset a daemon with running subprocesses."""
    command_factory('start')
    execute_add({'command': 'sleep 60'})
    execute_add({'command': 'sleep 60'})
    command_factory('reset')
    status = get_status()
    assert status['status'] == 'running'
    assert status['data'] == 'Queue is empty'
