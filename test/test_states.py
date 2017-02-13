from test.helper import (
    execute_add,
    command_factory,
)


def test_start(daemon_setup):
    """Start daemon after it has been paused."""
    command_factory('pause')()
    command_factory('start')()
    status = command_factory('status')()
    assert status['status'] == 'running'


def test_reset_paused(daemon_setup):
    """Reset the daemon."""
    command_factory('pause')()
    execute_add('sleep 60')
    execute_add('sleep 60')
    command_factory('reset')()
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['data'] == 'Queue is empty'


def test_reset_running(daemon_setup):
    """Reset a daemon with running subprocesses."""
    command_factory('start')()
    execute_add('sleep 60')
    execute_add('sleep 60')
    command_factory('reset')()
    status = command_factory('status')()
    assert status['status'] == 'running'
    assert status['data'] == 'Queue is empty'
