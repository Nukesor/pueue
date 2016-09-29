from test.helper import (
    execute_add,
    command_factory,
    get_status,
    send_command,
)


def test_remove_fails(daemon_setup):
    """Fail if removing a non existant key."""
    response = send_command({'mode': 'remove', 'key': 0})
    assert response['status'] == 'error'


def test_remove_running(daemon_setup):
    """Can't remove a running process."""
    execute_add({'command': 'sleep 60'})
    response = send_command({'mode': 'remove', 'key': 0})
    assert response['status'] == 'error'


def test_remove(daemon_setup):
    """Remove a process from the queue."""
    command_factory('pause')
    status = get_status()
    assert status['status'] == 'paused'
    execute_add({'command': 'ls'})

    response = send_command({'mode': 'remove', 'key': 0})
    assert response['status'] == 'success'
    status = get_status()
    assert status['data'] == 'Queue is empty'
