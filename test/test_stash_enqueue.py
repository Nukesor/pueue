from test.helper import (
    execute_add,
    command_factory,
)


def test_stash_enqueue(daemon_setup):
    """Kill a running process."""
    # Pause daemon to prevent the process to start
    command_factory('pause')()

    # Add process and stash it
    execute_add('sleep 60')
    command_factory('stash')({'keys': [0]})

    # Start daemon again
    command_factory('start')()

    # Check that it's stashed
    status = command_factory('status')()
    assert status['data'][0]['status'] == 'stashed'

    # Enqueue, start and ensure that its running
    command_factory('enqueue')({'keys': [0]})
    status = command_factory('status')()
    assert status['data'][0]['status'] == 'running'


def test_stash_running(daemon_setup):
    """Stash a running process."""
    # Add sleep entry
    execute_add('sleep 60')

    # Try to stash it, but it should fail
    status = command_factory('stash')({'keys': [0]})
    assert status['status'] == 'error'


def test_stash_paused(daemon_setup):
    """Stash a paused entry."""
    # Add process and pause it
    execute_add('sleep 60')
    command_factory('pause')()

    # Try to stash it, but it should fail
    status = command_factory('stash')({'keys': [0]})
    assert status['status'] == 'error'


def test_multiple_stash_enqueue(daemon_setup):
    """Stash and enqueue multiple valid entries."""
    # Pause daemon and add multiple entries
    command_factory('pause')()
    execute_add('ls')
    execute_add('sleep 1')
    execute_add('ls')

    # Stash all commands and ensure that they are stashed
    status = command_factory('stash')({'keys': [0, 1, 2]})
    assert status['status'] == 'success'

    status = command_factory('status')()
    assert status['data'][0]['status'] == 'stashed'
    assert status['data'][1]['status'] == 'stashed'
    assert status['data'][2]['status'] == 'stashed'

    # Enqueue all commands and ensure that they are queued
    status = command_factory('enqueue')({'keys': [0, 1, 2]})
    assert status['status'] == 'success'

    status = command_factory('status')()
    assert status['data'][0]['status'] == 'queued'
    assert status['data'][1]['status'] == 'queued'
    assert status['data'][2]['status'] == 'queued'


def test_multiple_stash_enqueue_invalid(daemon_setup):
    """Stash and enqueue multiple entries, but include invalid keys for enqueue."""
    # Pause daemon and add multiple entries
    command_factory('pause')()
    execute_add('ls')
    execute_add('sleep 1')
    execute_add('ls')

    # Stash all commands and ensure that they are stashed
    status = command_factory('stash')({'keys': [1, 2]})
    assert status['status'] == 'success'

    status = command_factory('status')()
    assert status['data'][0]['status'] == 'queued'
    assert status['data'][1]['status'] == 'stashed'
    assert status['data'][2]['status'] == 'stashed'

    # Enqueue all commands and ensure that they are queued
    # Response status should be `error` anyway, as we sent invalid keys
    status = command_factory('enqueue')({'keys': [0, 1, 2, 4]})
    assert status['status'] == 'error'

    status = command_factory('status')()
    assert status['data'][0]['status'] == 'queued'
    assert status['data'][1]['status'] == 'queued'
    assert status['data'][2]['status'] == 'queued'
