from test.helper import *
from test.fixtures import *


def test_remove_fails(daemon_setup):
    response = sendCommand({'mode': 'remove', 'key': 0})
    assert response['status'] == 'error'


def test_remove_running(daemon_setup):
    executeAdd({'command': 'sleep 60'})
    response = sendCommand({'mode': 'remove', 'key': 0})
    assert response['status'] == 'error'


def test_remove(daemon_setup):
    commandFactory('pause')
    status = getStatus()
    assert status['status'] == 'paused'
    executeAdd({'command': 'ls'})

    response = sendCommand({'mode': 'remove', 'key': 0})
    assert response['status'] == 'success'
    status = getStatus()
    assert '0' not in status['data']
