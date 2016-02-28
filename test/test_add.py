from test.helper import *
from test.fixtures import *


def test_add(daemon_setup):
    commandFactory('pause')
    response = sendCommand({
        'mode': 'add',
        'command': 'ls',
        'path': '/tmp',
        'status': 'queued',
        'returncode': ''
    })
    assert response['status'] == 'success'
    status = getStatus()
    assert status['data'][0]['command'] == 'ls'
    assert status['data'][0]['path'] == '/tmp'
