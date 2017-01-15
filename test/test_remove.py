from test.helper import (
    execute_add,
)
from test.helper import command_factory


def test_remove_fails(daemon_setup):
    """Fail if removing a non existant key."""
    response = command_factory('remove')({'key': 0})
    assert response['status'] == 'error'


def test_remove_running(daemon_setup):
    """Can't remove a running process."""
    execute_add({'command': 'sleep 60'})
    response = command_factory('remove')({'key': 0})
    assert response['status'] == 'error'


def test_remove(daemon_setup):
    """Remove a process from the queue."""
    command_factory('pause')()
    status = command_factory('status')()
    assert status['status'] == 'paused'
    execute_add({'command': 'ls'})

    response = command_factory('remove')({'key': 0})
    assert response['status'] == 'success'
    status = command_factory('status')()
    assert status['data'] == 'Queue is empty'
