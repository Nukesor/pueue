from test.helper import (
    execute_add,
    get_status,
    send_command,
    wait_for_process,
)


def test_restart_fails(daemon_setup):
    response = send_command({'mode': 'remove', 'key': 0})
    assert response['status'] == 'error'


def test_restart_running(daemon_setup):
    execute_add({'command': 'ls'})
    wait_for_process(0)
    response = send_command({'mode': 'restart', 'key': 0})
    assert response['status'] == 'success'
    status = get_status()
    assert len(status['data']) == 2
    assert status['data'][1]['path'] == status['data'][0]['path']
    assert status['data'][1]['command'] == status['data'][0]['command']
