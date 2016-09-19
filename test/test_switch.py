from test.helper import (
    command_factory,
    execute_add,
    get_status,
    send_command,
)


def test_switch(daemon_setup):
    command_factory('pause')
    execute_add({'command': 'ls'})
    execute_add({'command': 'ls -l'})
    command_factory('switch', {'first': 0, 'second': 1})
    status = get_status()
    assert status['data'][0]['command'] == 'ls -l'
    assert status['data'][1]['command'] == 'ls'


def test_switch_fails(daemon_setup):
    response = send_command({'mode': 'switch', 'first': 0, 'second': 1})
    assert response['status'] == 'error'


def test_switch_running(daemon_setup):
    execute_add({'command': 'sleep 60'})
    execute_add({'command': 'ls -l'})
    response = send_command({
        'mode': 'switch',
        'first': 0,
        'second': 1
    })
    assert response['status'] == 'error'
