from test.helper import (
    command_factory,
    get_status,
    send_command,
)


def test_add(daemon_setup):
    """The daemon adds a  new task to its queue."""
    command_factory('pause')
    response = send_command({
        'mode': 'add',
        'command': 'ls',
        'path': '/tmp',
        'status': 'queued',
        'returncode': ''
    })
    assert response['status'] == 'success'
    status = get_status()
    assert status['data'][0]['command'] == 'ls'
    assert status['data'][0]['path'] == '/tmp'
