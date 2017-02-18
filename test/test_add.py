from test.helper import command_factory


def test_add(daemon_setup):
    """The daemon adds a  new task to its queue."""
    response = command_factory('add')({
        'command': 'ls',
        'path': '/tmp',
    })
    assert response['status'] == 'success'
    status = command_factory('status')()
    assert status['data'][0]['command'] == 'ls'
    assert status['data'][0]['path'] == '/tmp'
