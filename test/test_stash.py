from test.helper import (
    execute_add,
    command_factory,
)


def test_stash(daemon_setup):
    """Kill a running process."""
    # Pause daemon to prevent the process to start
    command_factory('pause')()
    # Add process
    execute_add('sleep 60')
    # Stash it
    command_factory('stash')({'key': 0})
    # Check if it's stashed
    status = command_factory('status')()
    assert status['status'] == 'paused'
    assert status['data'][0]['status'] == 'stashed'

    # Enqueue, start and ensure that its running
    command_factory('enqueue')({'key': 0})
    command_factory('start')()
    status = command_factory('status')()
    assert status['data'][0]['status'] == 'running'


def test_stash_paused(daemon_setup):
    """Stash a paused process."""
    # Add process and pause it
    execute_add('sleep 60')
    command_factory('pause')()
    # Stash it, but it should fail
    status = command_factory('stash')({'key': 0})
    assert status['status'] == 'error'


def test_stash_running(daemon_setup):
    """Stash a running process."""
    # Add process and pause it
    execute_add('sleep 60')
    # Stash it, but it should fail
    status = command_factory('stash')({'key': 0})
    assert status['status'] == 'error'
