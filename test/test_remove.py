from test.helper import (
    execute_add,
    wait_for_process,
)
from test.helper import command_factory


def test_remove_fails(daemon_setup):
    """Fail if removing a non existant key."""
    response = command_factory('remove')({'keys': [0]})
    assert response['status'] == 'error'


def test_remove_running(daemon_setup):
    """Can't remove a running process."""
    execute_add('sleep 60')
    response = command_factory('remove')({'keys': [0]})
    assert response['status'] == 'error'


def test_remove(daemon_setup):
    """Remove a process from the queue."""
    # Pause the daemon. Otherwise we may try to remove a running entry.
    command_factory('pause')()

    # Add entry and instantly remove it.
    execute_add('ls')
    response = command_factory('remove')({'keys': [0]})
    assert response['status'] == 'success'

    # The queue should be empty
    status = command_factory('status')()
    assert status['data'] == 'Queue is empty'


def test_remove_multiple_specific_success(daemon_setup):
    """Remove various entries from the queue."""
    # Pause the daemon.
    command_factory('pause')()

    # Add entries
    execute_add('ls')
    execute_add('ls')
    execute_add('ls')

    # Remove two entries.
    response = command_factory('remove')({'keys': [0, 1]})
    assert response['status'] == 'success'

    status = command_factory('status')()
    assert 0 not in status['data']
    assert 1 not in status['data']


def test_remove_multiple_specific(daemon_setup):
    """Remove various entries from the queue."""
    # Pause the daemon.
    command_factory('pause')()

    # Add 4 entries to get a `failing`, `done`, `queued` and `running` entry.
    execute_add('failingtestcommand')
    execute_add('sleep 60')
    execute_add('ls')
    execute_add('ls')

    # Start 0, 1 and 2 and wait for the `failed` and `done` entry to finish.
    response = command_factory('start')({'keys': [0, 1, 2]})
    wait_for_process(0)
    status = wait_for_process(2)

    assert status['data'][0]['status'] == 'failed'
    assert status['data'][1]['status'] == 'running'
    assert status['data'][2]['status'] == 'done'
    assert status['data'][3]['status'] == 'queued'

    # Remove all entries. The response should be an error, as we try to remove
    # a running process.
    response = command_factory('remove')({'keys': [0, 1, 2, 3]})
    assert response['status'] == 'error'

    status = command_factory('status')()
    assert 0 not in status['data']
    assert status['data'][1]['status'] == 'running'
    assert 2 not in status['data']
    assert 3 not in status['data']
