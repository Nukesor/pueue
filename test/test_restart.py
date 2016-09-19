from test.helper import *
from test.fixtures import *


def test_restart_fails(daemon_setup):
    response = sendCommand({'mode': 'remove', 'key': 0})
    assert response['status'] == 'error'


def test_restart_running(daemon_setup):
    executeAdd({'command': 'ls'})
    response = sendCommand({'mode': 'restart', 'key': 0})
    status = getStatus()
    while status['process'] != 'No running process':
        status = getStatus()
    response = sendCommand({'mode': 'restart', 'key': 0})
    assert response['status'] == 'success'
    status = getStatus()
    assert len(status['data']) == 2
