from test.helper import *
from test.fixtures import *


def test_restart_fails(daemon_setup):
    response = send_command({'mode': 'remove', 'key': 0})
    assert response['status'] == 'error'


def test_restart_running(daemon_setup):
    execute_add({'command': 'ls'})
    response = send_command({'mode': 'restart', 'key': 0})
    status = get_status()
    while status['process'] != 'No running process':
        status = get_status()
    response = send_command({'mode': 'restart', 'key': 0})
    assert response['status'] == 'success'
    status = get_status()
    assert len(status['data']) == 2
