from test.helper import *
from test.fixtures import *


def test_switch(daemon_setup):
    commandFactory('pause')
    executeAdd({'command': 'ls'})
    executeAdd({'command': 'ls -l'})
    executeSwitch({'first': 0, 'second': 1})
    status = getStatus()
    assert status['data'][0]['command'] == 'ls -l'
    assert status['data'][1]['command'] == 'ls'


def test_switch_fails(daemon_setup):
    response = sendCommand({'mode': 'switch', 'first': 0, 'second': 1})
    assert response['status'] == 'error'


def test_switch_running(daemon_setup):
    executeAdd({'command': 'sleep 60'})
    executeAdd({'command': 'ls -l'})
    response = sendCommand({
        'mode': 'switch',
        'first': 0,
        'second': 1
    })
    assert response['status'] == 'error'
